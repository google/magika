# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Executable and bytecode validators; nothing is loaded, linked or run."""

from . import (
    beam,
    coff,
    dex,
    elf,
    javabytecode,
    llvm_bitcode,
    luabytecode,
    macho,
    odex,
    pe,
    pythonbytecode,
    spirv,
    threedsx,
    wasm,
    xcoff,
)

MODULES = (
    pe,
    elf,
    macho,
    coff,
    xcoff,
    dex,
    javabytecode,
    pythonbytecode,
    luabytecode,
    wasm,
    spirv,
    llvm_bitcode,
    beam,
    threedsx,
    odex,
)
