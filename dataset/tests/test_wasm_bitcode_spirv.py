import struct

from helpers import status

from magika_datasets.validators.executable import llvm_bitcode, spirv, wasm


def leb(value):
    out = b""
    while True:
        byte = value & 0x7F
        value >>= 7
        out += bytes([byte | (0x80 if value else 0)])
        if not value:
            return out


def section(identifier, payload):
    return bytes([identifier]) + leb(len(payload)) + payload


def module(trailing=b""):
    types = section(1, leb(1) + b"\x60" + leb(0) + leb(0))
    functions = section(3, leb(1) + leb(0))
    code = section(10, leb(1) + leb(2) + b"\x00\x0b")
    custom = section(0, leb(4) + b"name" + b"\x00")
    return b"\0asm\x01\0\0\0" + types + functions + code + custom + trailing


def test_wasm_sections():
    observation = wasm.validate(module(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "wasm")
    assert status(wasm, module()[:-2]) == "fail"
    assert (
        status(wasm, module(trailing=b"\x0c\x01\x00")) == "fail"
    )  # data count belongs before code
    assert status(wasm, module(trailing=b"\x0b\x01\x00")) == "pass"  # data section follows code
    assert status(wasm, module(trailing=b"\x99\x00")) == "fail"
    assert status(wasm, b"\0asm\x02\0\0\0") == "fail"
    assert status(wasm, b"\0asn" + b"\0" * 8) == "not_applicable"


def spirv_module(big=False, trailing=b""):
    order = ">" if big else "<"
    words = [0x07230203, 0x00010000, 0, 5, 0]
    words += [(2 << 16) | 17, 1]  # OpCapability Shader
    words += [(3 << 16) | 14, 0, 1]  # OpMemoryModel
    return b"".join(struct.pack(order + "I", w) for w in words) + trailing


def test_spirv_word_stream():
    observation = spirv.validate(spirv_module(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "spirv")
    assert status(spirv, spirv_module(big=True)) == "pass"
    assert status(spirv, spirv_module()[:-4]) == "fail"
    assert status(spirv, spirv_module(trailing=b"\0\0")) == "fail"
    assert status(spirv, spirv_module(trailing=b"\0\0\0\0")) == "fail"  # zero word count
    assert status(spirv, b"\x03\x02\x23\x07" + b"\0" * 16) == "fail"  # zero bound
    assert status(spirv, b"other") == "not_applicable"


class Bits:
    def __init__(self):
        self.bits = []

    def emit(self, value, width):
        for index in range(width):
            self.bits.append((value >> index) & 1)

    def vbr(self, value, width):
        threshold = 1 << (width - 1)
        while True:
            chunk = value & (threshold - 1)
            value >>= width - 1
            self.emit(chunk | (threshold if value else 0), width)
            if not value:
                return

    def align32(self):
        while len(self.bits) % 32:
            self.bits.append(0)

    def data(self):
        out = bytearray()
        for index in range(0, len(self.bits), 8):
            byte = 0
            for offset, bit in enumerate(self.bits[index : index + 8]):
                byte |= bit << offset
            out.append(byte)
        return bytes(out)


def bitcode(wrapped=False, corrupt_length=False):
    writer = Bits()
    for byte in b"BC\xc0\xde":
        writer.emit(byte, 8)
    # ENTER_SUBBLOCK (abbrev id 1, width 2), block id 8 (MODULE), new abbrev width 3, then length in words
    writer.emit(1, 2)
    writer.vbr(8, 8)
    writer.vbr(3, 4)
    writer.align32()
    length_position = len(writer.bits)
    writer.emit(0, 32)
    # inside: UNABBREV_RECORD (id 3, width 3): code 1 (version), 1 operand, value 2
    writer.emit(3, 3)
    writer.vbr(1, 6)
    writer.vbr(1, 6)
    writer.vbr(2, 6)
    # END_BLOCK (id 0, width 3)
    writer.emit(0, 3)
    writer.align32()
    words = (len(writer.bits) - length_position - 32) // 32
    for index in range(32):
        writer.bits[length_position + index] = ((words + (5 if corrupt_length else 0)) >> index) & 1
    data = writer.data()
    if wrapped:
        return struct.pack("<IIIII", 0x0B17C0DE, 0, 20, len(data), 0) + data
    return data


def test_llvm_bitcode_blocks():
    observation = llvm_bitcode.validate(bitcode(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "llvm_bitcode")
    wrapped = llvm_bitcode.validate(bitcode(wrapped=True), frozenset())
    assert wrapped.status == "pass" and "wrapped" in wrapped.tags
    assert status(llvm_bitcode, bitcode(corrupt_length=True)) == "fail"
    assert status(llvm_bitcode, bitcode()[:-4]) == "fail"
    assert status(llvm_bitcode, bitcode() + b"\0\0\0\0") == "fail"
    assert status(llvm_bitcode, b"BC\xc0\xdf" + b"\0" * 12) == "not_applicable"
