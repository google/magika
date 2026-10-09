# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""R serialized data: the XDR object stream walked item by item to the end of the file."""

import struct

from ..contract import Observation

FAMILY = "data"
FORMAT_IDS = ("rdata",)
SCOPE = "Uncompressed save() (RDX2/RDX3) and saveRDS() XDR streams: format version 2 or 3, writer and minimum R versions, native encoding, then every serialized item (pairlists, vectors, environments, references, byte code, ALTREP) walked to exactly the end of the file with reference indices checked; numeric payloads are not checksummed by the format; compressed files, ASCII and native-binary streams not interpreted"
ITEMS = 2_000_000

NIL, GLOBAL, UNBOUND, MISSING, BASENS, EMPTYENV, BASEENV = 254, 253, 252, 251, 250, 242, 241
REF, PERSIST, PACKAGE, NAMESPACE, ATTRLANG, ATTRLIST, ALTREP = 255, 247, 249, 248, 240, 239, 238
BCREPDEF, BCREPREF = 244, 243
SYM, LIST, CLOS, ENV, PROM, LANG, SPECIAL, BUILTIN, CHAR = 1, 2, 3, 4, 5, 6, 7, 8, 9
LGL, INT, REAL, CPLX, STR, DOT, VEC, EXPR, BCODE, EXTPTR, WEAKREF, RAW, S4 = (
    10,
    13,
    14,
    15,
    16,
    17,
    19,
    20,
    21,
    22,
    23,
    24,
    25,
)
SINGLETONS = {NIL, GLOBAL, UNBOUND, MISSING, BASENS, EMPTYENV, BASEENV}
PAIRS = {LIST, LANG, CLOS, PROM, DOT, ATTRLANG, ATTRLIST}
WIDTH = {LGL: 4, INT: 4, REAL: 8, CPLX: 16, RAW: 1}


class Stream:
    def __init__(self, data: bytes, offset: int):
        self.data, self.offset, self.refs, self.items = data, offset, 0, 0

    def integer(self) -> int:
        if self.offset + 4 > len(self.data):
            raise ValueError("Stream ended inside an item")
        value = struct.unpack_from(">i", self.data, self.offset)[0]
        self.offset += 4
        return value

    def skip(self, count: int) -> None:
        if count < 0 or self.offset + count > len(self.data):
            raise ValueError("Item data extends past the end of the file")
        self.offset += count

    def length(self) -> int:
        value = self.integer()
        if value == -1:
            upper, lower = self.integer(), self.integer()
            value = (upper << 32) + (lower & 0xFFFFFFFF)
        if value < 0:
            raise ValueError("Negative vector length")
        return value

    def strings(self) -> None:
        if self.integer() != 0:
            raise ValueError("String vector marker is not zero")
        for _ in range(self.length()):
            self.item()

    def item(self) -> None:
        # Pairlist tails are walked in a loop; only car and attributes recurse.
        while True:
            self.items += 1
            if self.items > ITEMS:
                raise OverflowError("Item budget exceeded")
            flags = self.integer()
            kind, has_attr, has_tag = flags & 0xFF, flags & (1 << 9), flags & (1 << 10)
            if kind in SINGLETONS:
                return
            if kind == REF:
                index = flags >> 8 or self.integer()
                if not 1 <= index <= self.refs:
                    raise ValueError(f"Reference {index} precedes its definition")
                return
            if kind in (PERSIST, PACKAGE, NAMESPACE):
                self.strings()
                self.refs += 1
                return
            if kind == SYM:
                self.item()
                self.refs += 1
                return
            if kind == ENV:
                self.refs += 1
                self.integer()  # locked
                for _ in range(4):  # enclosure, frame, hash table, attributes
                    self.item()
                return
            if kind in PAIRS:
                if has_attr:
                    self.item()
                if has_tag:
                    self.item()
                self.item()  # car
                continue  # cdr
            if kind == ALTREP:
                for _ in range(3):  # class info, state, attributes
                    self.item()
                return
            if kind == EXTPTR:
                self.refs += 1
                self.item()
                self.item()
            elif kind == WEAKREF:
                self.refs += 1
            elif kind in (SPECIAL, BUILTIN):
                self.skip(self.integer())
            elif kind == CHAR:
                size = self.integer()
                if size != -1:
                    self.skip(size)
                return  # R reads no attributes for CHARSXP
            elif kind in WIDTH:
                self.skip(self.length() * WIDTH[kind])
            elif kind in (STR, VEC, EXPR):
                for _ in range(self.length()):
                    self.item()
            elif kind == BCODE:
                self.integer()  # count of shared language cells
                self.bytecode()
            elif kind != S4:
                raise ValueError(f"Unknown item type {kind}")
            if has_attr:
                continue  # attributes are one more item
            return

    def bytecode(self) -> None:
        self.item()  # code vector
        for _ in range(self.integer()):
            kind = self.integer()
            if kind == BCODE:
                self.bytecode()
            elif kind in (LANG, LIST, BCREPDEF, BCREPREF, ATTRLANG, ATTRLIST):
                self.language(kind)
            else:
                self.item()

    def language(self, kind: int) -> None:
        while True:
            if kind == BCREPREF:
                self.integer()
                return
            if kind not in (BCREPDEF, LANG, LIST, ATTRLANG, ATTRLIST):
                self.item()
                return
            if kind == BCREPDEF:
                self.integer()
                kind = self.integer()
            if kind in (ATTRLANG, ATTRLIST):
                self.item()
            self.item()  # tag
            self.language(self.integer())  # car
            kind = self.integer()  # cdr


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith((b"RDX2\nX\n", b"RDX3\nX\n")):
        start, container = 7, "save()"
    elif data.startswith(b"X\n\0\0\0\x02") or data.startswith(b"X\n\0\0\0\x03"):
        start, container = 2, "saveRDS()"
    else:
        return None
    stream = Stream(data, start)
    try:
        version = stream.integer()
        writer, minimum = stream.integer(), stream.integer()
        if version not in (2, 3) or (container == "save()" and data[3] - 0x30 != version):
            raise ValueError("Format version does not match the file header")
        if not 0x020000 <= minimum <= writer < 0x0A0000:
            raise ValueError("Writer or minimum R version implausible")
        encoding = ""
        if version == 3:
            size = stream.integer()
            if not 0 < size <= 64:
                raise ValueError("Native encoding name length invalid")
            encoding = data[stream.offset : stream.offset + size].decode("ascii")
            stream.skip(size)
        stream.item()
        if stream.offset != len(data):
            raise ValueError(f"{len(data) - stream.offset} bytes follow the serialized object")
    except OverflowError as error:
        return Observation("inconclusive", str(error), "rdata")
    except (ValueError, UnicodeDecodeError, RecursionError) as error:
        return Observation("fail", str(error) or "Malformed stream", "rdata")
    release = f"{writer >> 16}.{writer >> 8 & 0xFF}.{writer & 0xFF}"
    return Observation(
        "pass",
        f"{container} format {version} from R {release}"
        + (f" ({encoding})" if encoding else "")
        + f"; {stream.items} items walked to the end of the file",
        "rdata",
    )
