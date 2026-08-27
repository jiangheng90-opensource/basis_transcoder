#pragma once
#include "rust/cxx.h"
#include <cstdint>
#include <memory>

// Forward declaration - defined in ffi.rs.h
struct TextureInfo;

// Forward declaration for basist::ktx2_transcoder
namespace basist {
class ktx2_transcoder;
}

// Opaque wrapper around basist::ktx2_transcoder.
class Ktx2Transcoder {
public:
  std::unique_ptr<basist::ktx2_transcoder> transcoder;

  explicit Ktx2Transcoder(std::unique_ptr<basist::ktx2_transcoder> t);
  ~Ktx2Transcoder();
};

// Creates a transcoder from KTX2 Basis Universal data and starts transcoding.
// Returns nullptr on failure.
std::unique_ptr<Ktx2Transcoder> create_transcoder(rust::Slice<const uint8_t> data);

// Queries basic texture information. Returns false on failure.
bool get_texture_info(const Ktx2Transcoder &transcoder, TextureInfo &out);

// Computes the tightly-packed output size in bytes for one mip level
// (2D textures: layer 0, face 0) transcoded to `format`
// (basist::transcoder_texture_format). Returns 0 on failure.
size_t get_level_output_size(const Ktx2Transcoder &transcoder, uint32_t level, uint32_t format,
                             uint32_t &out_width, uint32_t &out_height);

// Transcodes one mip level (2D textures: layer 0, face 0) into the
// pre-allocated `out` buffer. `out_len` must be the size returned by
// get_level_output_size. Returns false on failure.
bool transcode_level(const Ktx2Transcoder &transcoder, uint32_t level, uint32_t format,
                     uint8_t *out_ptr, size_t out_len);
