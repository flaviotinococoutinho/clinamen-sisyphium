/* Independent C11 ABI consumer (REQ-H-03). */
#include "sisyphium.h"
#include "fixture.h"
#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

_Static_assert(sizeof(SyFacultates) == 16, "SyFacultates size");
_Static_assert(offsetof(SyFacultates, width) == 0, "SyFacultates.width");
_Static_assert(offsetof(SyFacultates, height) == 2, "SyFacultates.height");
_Static_assert(offsetof(SyFacultates, color_bits) == 4, "SyFacultates.color_bits");
_Static_assert(offsetof(SyFacultates, max_hz) == 5, "SyFacultates.max_hz");
_Static_assert(offsetof(SyFacultates, touch) == 6, "SyFacultates.touch");
_Static_assert(offsetof(SyFacultates, motion) == 7, "SyFacultates.motion");
_Static_assert(offsetof(SyFacultates, occlusion) == 8, "SyFacultates.occlusion");
_Static_assert(offsetof(SyFacultates, persistence) == 9, "SyFacultates.persistence");
_Static_assert(offsetof(SyFacultates, cells) == 10, "SyFacultates.cells");
_Static_assert(offsetof(SyFacultates, australis) == 11, "SyFacultates.australis");
_Static_assert(offsetof(SyFacultates, tz_offset_min) == 12, "SyFacultates.tz_offset_min");
_Static_assert(offsetof(SyFacultates, lingua) == 14, "SyFacultates.lingua");
_Static_assert(offsetof(SyFacultates, compute) == 15, "SyFacultates.compute");
_Static_assert(sizeof(SyIndicia) == 32, "SyIndicia size");
_Static_assert(offsetof(SyIndicia, epoch_ms) == 0, "SyIndicia.epoch_ms");
_Static_assert(offsetof(SyIndicia, energy) == 8, "SyIndicia.energy");
_Static_assert(offsetof(SyIndicia, seism) == 9, "SyIndicia.seism");
_Static_assert(offsetof(SyIndicia, automatism) == 10, "SyIndicia.automatism");
_Static_assert(offsetof(SyIndicia, reservatum0) == 11, "SyIndicia.reservatum0");
_Static_assert(offsetof(SyIndicia, idle_s) == 12, "SyIndicia.idle_s");
_Static_assert(offsetof(SyIndicia, returned_ms) == 16, "SyIndicia.returned_ms");
_Static_assert(offsetof(SyIndicia, occluded) == 20, "SyIndicia.occluded");
_Static_assert(offsetof(SyIndicia, leaving) == 21, "SyIndicia.leaving");
_Static_assert(offsetof(SyIndicia, touch_x) == 22, "SyIndicia.touch_x");
_Static_assert(offsetof(SyIndicia, touch_y) == 23, "SyIndicia.touch_y");
_Static_assert(offsetof(SyIndicia, has_touch) == 24, "SyIndicia.has_touch");
_Static_assert(offsetof(SyIndicia, offset) == 25, "SyIndicia.offset");
_Static_assert(offsetof(SyIndicia, tilt) == 26, "SyIndicia.tilt");
_Static_assert(offsetof(SyIndicia, reservata) == 27, "SyIndicia.reservata");
_Static_assert(sizeof(SyRegimen) == 16, "SyRegimen size");
_Static_assert(offsetof(SyRegimen, stilus) == 0, "SyRegimen.stilus");
_Static_assert(offsetof(SyRegimen, strata) == 1, "SyRegimen.strata");
_Static_assert(offsetof(SyRegimen, reservata0) == 2, "SyRegimen.reservata0");
_Static_assert(offsetof(SyRegimen, tick_ms) == 4, "SyRegimen.tick_ms");
_Static_assert(offsetof(SyRegimen, interval_ms) == 8, "SyRegimen.interval_ms");
_Static_assert(offsetof(SyRegimen, layers) == 12, "SyRegimen.layers");
_Static_assert(offsetof(SyRegimen, lines) == 13, "SyRegimen.lines");
_Static_assert(offsetof(SyRegimen, reservata1) == 14, "SyRegimen.reservata1");
_Static_assert(_Alignof(SyFacultates) == 2, "Facultates alignment");
_Static_assert(_Alignof(SyRegimen) == 4, "Regimen alignment");
_Static_assert(_Alignof(SyIndicia) == _Alignof(int64_t), "Indicia native alignment");
#if UINTPTR_MAX == UINT64_MAX
_Static_assert(_Alignof(SyIndicia) == 8, "Indicia CI64 alignment");
#endif

/* Confirms the exact persisted core fixture and adjacent sentinels (REQ-H-03). */
static void check_words(uint64_t word, uint64_t memory, const uint8_t expected[32]) {
    uint8_t text[34];
    memset(text, 0xa5, sizeof text);
    sisyphium_format(word, memory, text+1);
    assert(text[0] == 0xa5 && text[33] == 0xa5);
    assert(memcmp(text+1, expected, 32) == 0);
}

/* Executes every export, checking core fixtures and rejected calls (REQ-H-03). */
int main(void) {
    const SyFacultates f = {24,16,8,24,0,0,0,0,0,1,-180,0,255};
    const SyIndicia i = {INT64_C(1700000000000),255,0,0,0,UINT32_MAX,UINT32_MAX,0,0,0,0,0,128,0,{0,0,0,0,0}};
    SyRegimen r;
    uint64_t word=0, memory=0, parsed_word=0, parsed_memory=0;
    uint8_t pixels[386], palette[50], text[34];
    assert(sisyphium_version() == EXPECTED_VERSION);
    sisyphium_genesis(123, &word, &memory);
    check_words(word,memory,GENESIS);
    memset(text,0xa5,sizeof text);
    sisyphium_format(word,memory,text+1);
    assert(sisyphium_parse(text+1,&parsed_word,&parsed_memory) == 1);
    assert(word == parsed_word && memory == parsed_memory);
    memset(&r,0xa5,sizeof r);
    sisyphium_consilium(&f,word,memory,255,&r);
    assert(memcmp(&r,&REGIMEN,sizeof r) == 0);
    memset(pixels,0xa5,sizeof pixels);
    memset(palette,0xa5,sizeof palette);
    sisyphium_render(word,memory,&f,&r,&i,pixels+1,24,16);
    sisyphium_palette(word,memory,&f,&r,&i,palette+1);
    assert(memcmp(pixels+1,PIXELS,384) == 0);
    assert(memcmp(palette+1,PALETTE,48) == 0);
    assert(pixels[0] == 0xa5 && pixels[385] == 0xa5);
    assert(palette[0] == 0xa5 && palette[49] == 0xa5);
    assert(sisyphium_step(&word,&memory,&f,&i) == STEP);
    check_words(word,memory,STEPPED);
    assert(sisyphium_advance(&word,&memory,&f,&i,33,r.tick_ms) == ADVANCE);
    check_words(word,memory,ADVANCED);
    {
        const uint64_t before_word=word, before_memory=memory;
        SyFacultates bad=f;
        bad.lingua=255;
        assert(sisyphium_step(&word,&memory,&bad,&i) == 0);
        assert(sisyphium_advance(&word,&memory,&f,&i,65537,42) == 0);
        text[1]='z';
        assert(sisyphium_parse(text+1,&word,&memory) == 0);
        assert(sisyphium_parse(NULL,&word,&memory) == 0);
        sisyphium_genesis(1,NULL,&memory);
        sisyphium_genesis(1,&word,NULL);
        assert(word == before_word && memory == before_memory);
        memset(palette,0xa5,sizeof palette);
        r.reservata1[1]=1;
        sisyphium_palette(word,memory,&f,&r,&i,palette+1);
        for (size_t index=0; index<sizeof palette; ++index) assert(palette[index] == 0xa5);
    }
    puts("ffi: nine exports, layouts, core fixtures and sentinels passed");
    return 0;
}
