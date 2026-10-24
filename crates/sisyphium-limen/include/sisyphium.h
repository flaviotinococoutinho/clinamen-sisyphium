/* C11 boundary of the 128-bit world (REQ-H-03; docs/04-arquitetura.md section 6).
 * Non-null pointers must be aligned, live for the entire call and refer to initialized
 * readable inputs or writable outputs of the documented extent. Distinct outputs must
 * not overlap each other or any input. No concurrent conflicting access is allowed.
 * Null is rejected, but null checks cannot validate arbitrary non-null addresses.
 * Invalid parameters: void calls do nothing; parse/step/advance return zero without writes.
 * Step/advance zero is also a valid empty announcement, not an unambiguous error code.
 * No state is retained. No function owns, frees or retains caller memory.
 */
#ifndef SISYPHIUM_H
#define SISYPHIUM_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct SyFacultates {
    uint16_t width; /* Frame width, positive. */
    uint16_t height; /* Frame height, positive. */
    uint8_t color_bits; /* Color depth: 1, 4, 8 or 24. */
    uint8_t max_hz; /* Maximum presentation rate; zero means static. */
    uint8_t touch; /* Boolean capability, exactly zero or one. */
    uint8_t motion; /* Boolean capability, exactly zero or one. */
    uint8_t occlusion; /* Boolean capability, exactly zero or one. */
    uint8_t persistence; /* Boolean capability, exactly zero or one. */
    uint8_t cells; /* Boolean capability, exactly zero or one. */
    uint8_t australis; /* Boolean capability, exactly zero or one. */
    int16_t tz_offset_min; /* UTC offset in minutes. */
    uint8_t lingua; /* Language: English zero, Portuguese one. */
    uint8_t compute; /* Relative compute budget. */
} SyFacultates;

typedef struct SyIndicia {
    int64_t epoch_ms; /* UTC epoch in milliseconds. */
    uint8_t energy; /* Available energy. */
    uint8_t seism; /* External agitation. */
    uint8_t automatism; /* Interaction regularity. */
    uint8_t reservatum0; /* Reserved; must be zero. */
    uint32_t idle_s; /* Idle seconds; maximum means unknown. */
    uint32_t returned_ms; /* Time since return; maximum means never. */
    uint8_t occluded; /* Occluded flag, zero or one. */
    uint8_t leaving; /* Leaving flag, zero or one. */
    uint8_t touch_x; /* Normalized horizontal touch. */
    uint8_t touch_y; /* Normalized vertical touch. */
    uint8_t has_touch; /* Touch present flag, zero or one. */
    uint8_t offset; /* Horizontal launcher offset. */
    int8_t tilt; /* Device roll. */
    uint8_t reservata[5]; /* Reserved; all bytes must be zero. */
} SyIndicia;

typedef struct SyRegimen {
    uint8_t stilus; /* Visual idiom: Creta zero through Bicolor three. */
    uint8_t strata; /* Parallax strata, 1 through 4. */
    uint8_t reservata0[2]; /* Reserved; all bytes must be zero. */
    uint32_t tick_ms; /* Tick cadence: 42, 125, 250 or 60000 ms. */
    uint32_t interval_ms; /* Presentation interval; 42, 125, 250, 1000 or 60000; at least tick_ms. */
    uint8_t layers; /* Enabled layer bits. */
    uint8_t lines; /* Chronicle lines, zero through six. */
    uint8_t reservata1[2]; /* Reserved; all bytes must be zero. */
} SyRegimen;

/* Version: major<<16 | minor<<8 | patch, each component at most 255. */
uint32_t sisyphium_version(void);
void sisyphium_genesis(uint64_t seed, uint64_t *verbum, uint64_t *memoria);
/* Reads exactly 32 bytes; outputs are two disjoint uint64_t objects. */
int32_t sisyphium_parse(const uint8_t text[32], uint64_t *verbum, uint64_t *memoria);
/* Writes exactly 32 bytes; no NUL is appended. */
void sisyphium_format(uint64_t verbum, uint64_t memoria, uint8_t out[32]);
void sisyphium_consilium(const SyFacultates *f, uint64_t verbum, uint64_t memoria, uint8_t energy, SyRegimen *out);
/* Both state words are initialized readable AND writable uint64_t objects. */
uint32_t sisyphium_step(uint64_t *verbum, uint64_t *memoria, const SyFacultates *f, const SyIndicia *i);
/* ticks <= 65536; tick_ms is 42, 125, 250 or 60000. */
uint32_t sisyphium_advance(uint64_t *verbum, uint64_t *memoria, const SyFacultates *f, const SyIndicia *i, uint32_t ticks, uint32_t tick_ms);
/* width/height equal f; pixels contains width*height writable bytes in one allocation.
 * Pixels may initially be uninitialized; the call initializes the output before rendering.
 * Extent must fit size_t and be <= PTRDIFF_MAX (Rust isize::MAX). */
void sisyphium_render(uint64_t verbum, uint64_t memoria, const SyFacultates *f, const SyRegimen *r, const SyIndicia *i, uint8_t *pixels, uint16_t width, uint16_t height);
/* Writes exactly 48 bytes: sixteen contiguous RGB triples. No NUL is appended. */
void sisyphium_palette(uint64_t verbum, uint64_t memoria, const SyFacultates *f, const SyRegimen *r, const SyIndicia *i, uint8_t out[48]);
#ifdef __cplusplus
}
#endif
#endif
