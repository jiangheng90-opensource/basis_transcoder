use js_sys::{Array, Promise, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::utils::{LevelData, TargetFormat, TextureInfo, TranscodedTexture};

thread_local! {
    static TRANSCODE_FUNC_MODULE: std::cell::RefCell<Option<JsValue>> = std::cell::RefCell::new(None);
    static BASIS_CORE_MODULE: std::cell::RefCell<Option<JsValue>> = std::cell::RefCell::new(None);
}

/// Imports an embedded JS bundle by eval'ing it into a blob module.
///
/// Works in any wasm context (main thread or a dedicated worker), which is
/// what lets the pure-transcode path run inside a host's own worker.
async fn import_bundle(code: &'static str) -> Result<JsValue, JsValue> {
    let escaped = code.replace("\\", "\\\\").replace("`", "\\`");

    let setup_code = format!(
        r#"
        (function() {{
            const code = `{escaped}`;
            const blob = new Blob([code], {{ type: "application/javascript" }});
            const url = URL.createObjectURL(blob);
            return import(url).then(mod => {{
                URL.revokeObjectURL(url);
                return mod;
            }});
        }})()
    "#
    );

    let js_module = js_sys::eval(&setup_code)?;
    let module_promise: Promise = js_module.dyn_into()?;
    JsFuture::from(module_promise).await
}

/// The worker-hosted bundle (`javascript/index.es.js`): transcoding runs in
/// the bundle's own dedicated worker.
async fn get_js_module() -> Result<JsValue, JsValue> {
    if let Some(module) = TRANSCODE_FUNC_MODULE.with(|m| m.borrow().clone()) {
        return Ok(module);
    }

    let module = import_bundle(include_str!("../javascript/index.es.js")).await?;
    TRANSCODE_FUNC_MODULE.with(|m| m.replace(Some(module.clone())));

    Ok(module)
}

/// The pure-transcode bundle (`javascript/core.es.js`): transcoding runs in
/// the CURRENT context — no worker is spawned.
async fn get_core_js_module() -> Result<JsValue, JsValue> {
    if let Some(module) = BASIS_CORE_MODULE.with(|m| m.borrow().clone()) {
        return Ok(module);
    }

    let module = import_bundle(include_str!("../javascript/core.es.js")).await?;
    BASIS_CORE_MODULE.with(|m| m.replace(Some(module.clone())));

    Ok(module)
}

fn get_u32(obj: &JsValue, key: &str) -> Result<u32, JsValue> {
    Ok(js_sys::Reflect::get(obj, &JsValue::from_str(key))?
        .as_f64()
        .unwrap_or(0.0) as u32)
}

/// Parses the transcode result object `{ width, ..., mips: [...] }` into a
/// [`TranscodedTexture`]. Shared by the worker-hosted and local paths.
fn parse_transcoded_texture(
    out_obj: &JsValue,
    target: TargetFormat,
) -> Result<TranscodedTexture, JsValue> {
    let info = TextureInfo {
        width: get_u32(out_obj, "width")?,
        height: get_u32(out_obj, "height")?,
        levels: get_u32(out_obj, "levels")?,
        faces: get_u32(out_obj, "faces")?,
        layers: get_u32(out_obj, "layers")?,
        has_alpha: get_u32(out_obj, "hasAlpha")? != 0,
        is_etc1s: get_u32(out_obj, "isEtc1s")? != 0,
        is_uastc: get_u32(out_obj, "isUastc")? != 0,
        is_srgb: get_u32(out_obj, "isSrgb")? != 0,
        is_hdr: get_u32(out_obj, "isHdr")? != 0,
        is_video: get_u32(out_obj, "isVideo")? != 0,
    };

    let mips_array =
        js_sys::Reflect::get(out_obj, &JsValue::from_str("mips"))?.dyn_into::<Array>()?;

    let mut levels = Vec::with_capacity(mips_array.length() as usize);
    for i in 0..mips_array.length() {
        let mip_obj = mips_array.get(i);
        let data =
            js_sys::Reflect::get(&mip_obj, &JsValue::from_str("data"))?.dyn_into::<Uint8Array>()?;
        levels.push(LevelData {
            width: get_u32(&mip_obj, "width")?,
            height: get_u32(&mip_obj, "height")?,
            data: data.to_vec(),
        });
    }

    Ok(TranscodedTexture {
        info,
        format: target,
        levels,
    })
}

/// Calls `transcodeKtx2InWorker` on the worker-hosted bundle.
async fn transcode_ktx2_from_embedded_js(
    data: &Uint8Array,
    target: TargetFormat,
) -> Result<TranscodedTexture, JsValue> {
    let module = get_js_module().await?;

    let transcode_fn = js_sys::Reflect::get(&module, &JsValue::from_str("transcodeKtx2InWorker"))?
        .dyn_into::<js_sys::Function>()?;

    let this = JsValue::NULL;
    let result = transcode_fn.call2(&this, data, &JsValue::from_f64(target.as_u32() as f64))?;
    let transcode_promise: Promise = result.dyn_into()?;
    let out_obj = JsFuture::from(transcode_promise).await?;

    parse_transcoded_texture(&out_obj, target)
}

/// Calls `transcodeKtx2Local` on the pure-transcode bundle, running the
/// transcode in the current context (no worker round-trip).
async fn transcode_ktx2_from_core_js(
    data: &Uint8Array,
    target: TargetFormat,
) -> Result<TranscodedTexture, JsValue> {
    let module = get_core_js_module().await?;

    let transcode_fn = js_sys::Reflect::get(&module, &JsValue::from_str("transcodeKtx2Local"))?
        .dyn_into::<js_sys::Function>()?;

    let this = JsValue::NULL;
    let result = transcode_fn.call2(&this, data, &JsValue::from_f64(target.as_u32() as f64))?;
    let transcode_promise: Promise = result.dyn_into()?;
    let out_obj = JsFuture::from(transcode_promise).await?;

    parse_transcoded_texture(&out_obj, target)
}

pub async fn transcode_ktx2_wasm_worker(
    data: &[u8],
    target: TargetFormat,
) -> Option<TranscodedTexture> {
    let js_array = Uint8Array::from(data);

    match transcode_ktx2_from_embedded_js(&js_array, target).await {
        Ok(texture) => Some(texture),
        Err(err) => {
            web_sys::console::error_1(&err);
            None
        }
    }
}

pub async fn transcode_ktx2_local(data: &[u8], target: TargetFormat) -> Option<TranscodedTexture> {
    let js_array = Uint8Array::from(data);

    match transcode_ktx2_from_core_js(&js_array, target).await {
        Ok(texture) => Some(texture),
        Err(err) => {
            web_sys::console::error_1(&err);
            None
        }
    }
}
