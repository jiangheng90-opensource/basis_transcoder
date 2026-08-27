//! Decodes a KTX2 Basis Universal texture to PNG images (one per mip level)
//! for visual verification of the transcoder output.
//!
//! Usage: cargo run --example decode_to_png -- <input.ktx2> [output_prefix]
//!
//! PNGs are written to the gitignored `output/` directory.

use basis_transcoder::{TargetFormat, transcode_ktx2_sync};

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().unwrap_or_else(|| {
        eprintln!("usage: decode_to_png <input.ktx2> [output_prefix]");
        std::process::exit(2);
    });
    let prefix = args.next().unwrap_or_else(|| {
        std::path::Path::new(&input)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "out".to_string())
    });

    let data = std::fs::read(&input).unwrap_or_else(|e| panic!("failed to read {input}: {e}"));

    let texture = transcode_ktx2_sync(&data, TargetFormat::Rgba32)
        .unwrap_or_else(|| panic!("failed to transcode {input}"));

    println!(
        "{input}: {}x{}, levels={}, faces={}, layers={}, alpha={}, etc1s={}, uastc={}, srgb={}",
        texture.info.width,
        texture.info.height,
        texture.info.levels,
        texture.info.faces,
        texture.info.layers,
        texture.info.has_alpha,
        texture.info.is_etc1s,
        texture.info.is_uastc,
        texture.info.is_srgb,
    );

    let out_dir = std::path::Path::new("output");
    std::fs::create_dir_all(out_dir).expect("failed to create output directory");

    for (index, level) in texture.levels.iter().enumerate() {
        let path = out_dir.join(format!("{prefix}_level{index}.png"));
        image::save_buffer(
            &path,
            &level.data,
            level.width,
            level.height,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
        println!("wrote {} ({}x{})", path.display(), level.width, level.height);
    }
}
