// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! System V Executable and Linkable Format (`ELF`) metaformat verifiers (`elf`, `cubin`).

use crate::ContentType as Label;
use crate::resolvers::magic::*;

/// `EM_CUDA` (`190`) in the endianness declared by `EI_DATA` at offset 5.
pub(crate) fn cubin(prefix: &[u8]) -> Option<Label> {
    const ELF_MAGIC: &[u8] = b"\x7fELF"; // @0..4: ELF magic
    const EI_DATA: usize = 5; // @5: u8 — 1 = ELFDATA2LSB, 2 = ELFDATA2MSB
    const E_MACHINE: usize = 18; // @18: u16 — ELF e_machine field
    const EM_CUDA: u16 = 190; // NVIDIA CUDA architecture
    const MIN_LEN: usize = 20;

    require!(prefix.len() >= MIN_LEN, starts(prefix, ELF_MAGIC));
    let is_cuda = match u8_at(prefix, EI_DATA)? {
        1 => le16(prefix, E_MACHINE)? == EM_CUDA,
        2 => be16(prefix, E_MACHINE)? == EM_CUDA,
        _ => false,
    };
    require!(is_cuda);
    Some(Label::Cubin)
}

/// System V / Linux Executable and Linkable Format (`elf`, 1995): `\x7FELF` at `@0`,
/// `EI_CLASS` `1|2` at `@4`, `EI_DATA` `1|2` at `@5`, `EI_VERSION == 1` at `@6`,
/// and valid `e_type` (`1..=4` or `0xFE00..=0xFFFF`) at `@16`.
pub(crate) fn elf(prefix: &[u8], size: u64) -> Option<Label> {
    const MIN_ELF_SIZE: u64 = 52; // 52B ELF32 header (64B for ELF64)
    const ELF_MAGIC: &[u8; 4] = b"\x7FELF";

    require!(
        size >= MIN_ELF_SIZE,
        prefix.len() >= 52,
        starts(prefix, ELF_MAGIC),
        matches!(u8_at(prefix, 4)?, 1 | 2),
        matches!(u8_at(prefix, 5)?, 1 | 2),
        u8_at(prefix, 6)? == 1,
    );
    let is_le = u8_at(prefix, 5)? == 1;
    let e_type = if is_le { le16(prefix, 16)? } else { be16(prefix, 16)? };
    require!((1..=4).contains(&e_type) || e_type >= 0xFE00);
    Some(Label::Elf)
}

/// Disambiguates System V ELF binaries between NVIDIA CUDA `cubin` and generic `elf`.
#[inline]
pub(crate) fn cubin_or_elf(prefix: &[u8], size: u64) -> Option<Label> {
    probe(|| cubin(prefix)).or_else(|| probe(|| elf(prefix, size)))
}
