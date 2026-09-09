/* Copyright 2026 Google LLC
 * SPDX-License-Identifier: Apache-2.0 */
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
uint32_t Crc32c_ComputeBuf(uint32_t, const void *, size_t);
static uint32_t oracle(uint32_t crc, const unsigned char *p, size_t n) {
    while (n--) {
        crc ^= *p++;
        for (unsigned i = 0; i < 8; ++i)
            crc = (crc >> 1) ^ (0x82f63b78u & (0u - (crc & 1)));
    }
    return crc;
}
int main(void) {
    unsigned char *p = malloc(1700000 + 32);
    assert(p);
    uint32_t x = 0x15ad8923;
    for (size_t i = 0; i < 1700000 + 32; ++i) {
        x ^= x << 13; x ^= x >> 17; x ^= x << 5; p[i] = x;
    }
    const uint32_t seeds[] = {0, 1, 0xffffffffu, 0xabcdef98};
    unsigned cases = 0;
    for (unsigned s = 0; s < 4; ++s)
        for (unsigned offset = 0; offset < 32; ++offset)
            for (size_t len = 0; len <= 512; ++len) {
                assert(Crc32c_ComputeBuf(seeds[s], p + offset, len)
                       == oracle(seeds[s], p + offset, len)); ++cases;
            }
    for (unsigned offset = 0; offset < 32; ++offset) {
        assert(Crc32c_ComputeBuf(0xffffffffu, p + offset, 1600003)
               == oracle(0xffffffffu, p + offset, 1600003)); ++cases;
    }
    free(p);
    printf("{\"crc32c_oracle_cases\":%u,\"passed\":true}\n", cases);
}
