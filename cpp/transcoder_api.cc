#include "transcoder_api.h"
#include "basis_transcoder/src/ffi.rs.h"

#include "basisu_transcoder.h"

Ktx2Transcoder::Ktx2Transcoder(std::unique_ptr<basist::ktx2_transcoder> t)
    : transcoder(std::move(t)) {}
Ktx2Transcoder::~Ktx2Transcoder() = default;

std::unique_ptr<Ktx2Transcoder> create_transcoder(rust::Slice<const uint8_t> data) {
  if (data.size() == 0 || data.size() > UINT32_MAX) {
    return nullptr;
  }

  // Idempotent; safe to call for every file.
  basist::basisu_transcoder_init();

  auto transcoder = std::make_unique<basist::ktx2_transcoder>();
  if (!transcoder->init(data.data(), static_cast<uint32_t>(data.size()))) {
    return nullptr;
  }
  if (!transcoder->start_transcoding()) {
    return nullptr;
  }
  return std::make_unique<Ktx2Transcoder>(std::move(transcoder));
}

bool get_texture_info(const Ktx2Transcoder &transcoder, TextureInfo &out) {
  const basist::ktx2_transcoder &t = *transcoder.transcoder;
  out.width = t.get_width();
  out.height = t.get_height();
  out.levels = t.get_levels();
  out.faces = t.get_faces();
  out.layers = t.get_layers();
  out.has_alpha = t.get_has_alpha();
  out.is_etc1s = t.is_etc1s() ? 1 : 0;
  out.is_uastc = t.is_uastc() ? 1 : 0;
  out.is_srgb = t.is_srgb() ? 1 : 0;
  out.is_hdr = t.is_hdr() ? 1 : 0;
  out.is_video = t.is_video() ? 1 : 0;
  return out.width > 0 && out.height > 0 && out.levels > 0;
}

size_t get_level_output_size(const Ktx2Transcoder &transcoder, uint32_t level, uint32_t format,
                             uint32_t &out_width, uint32_t &out_height) {
  basist::ktx2_image_level_info level_info;
  if (!transcoder.transcoder->get_image_level_info(level_info, level, 0, 0)) {
    return 0;
  }

  const auto fmt = static_cast<basist::transcoder_texture_format>(format);
  const uint32_t bytes_per_block_or_pixel = basist::basis_get_bytes_per_block_or_pixel(fmt);
  if (bytes_per_block_or_pixel == 0) {
    return 0;
  }

  out_width = level_info.m_orig_width;
  out_height = level_info.m_orig_height;

  if (basist::basis_transcoder_format_is_uncompressed(fmt)) {
    return static_cast<size_t>(level_info.m_orig_width) * level_info.m_orig_height *
           bytes_per_block_or_pixel;
  }

  const uint32_t block_width = basist::basis_get_block_width(fmt);
  const uint32_t block_height = basist::basis_get_block_height(fmt);
  const uint32_t blocks_x = (level_info.m_width + block_width - 1) / block_width;
  const uint32_t blocks_y = (level_info.m_height + block_height - 1) / block_height;
  return static_cast<size_t>(blocks_x) * blocks_y * bytes_per_block_or_pixel;
}

bool transcode_level(const Ktx2Transcoder &transcoder, uint32_t level, uint32_t format,
                     uint8_t *out_ptr, size_t out_len) {
  if (out_ptr == nullptr || out_len == 0 || out_len > UINT32_MAX) {
    return false;
  }

  const auto fmt = static_cast<basist::transcoder_texture_format>(format);
  const uint32_t bytes_per_block_or_pixel = basist::basis_get_bytes_per_block_or_pixel(fmt);
  if (bytes_per_block_or_pixel == 0) {
    return false;
  }

  // output_blocks_buf_size is in blocks for compressed formats, in pixels otherwise.
  const auto out_size = static_cast<uint32_t>(out_len);
  const uint32_t blocks_or_pixels = out_size / bytes_per_block_or_pixel;
  if (blocks_or_pixels * bytes_per_block_or_pixel != out_size) {
    return false;
  }

  return transcoder.transcoder->transcode_image_level(level, 0, 0, out_ptr, blocks_or_pixels,
                                                      fmt);
}
