# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Compound File Binary containers walked once; stream names name the Office format."""

import re
import struct
import uuid

from ..contract import Observation

FAMILY = "office"
FORMAT_IDS = ("ole", "doc", "xls", "ppt", "msi", "outlook", "thumbsdb", "hwp", "visio")
SHARED_FORMAT_IDS = ("visio",)  # OPC .vsdx packages are named by archive/zip.py
SCOPE = "CFB header, DIFAT and FAT sectors inside the file, directory chain, every stream's FAT or mini FAT chain long enough for its size without loops, root storage stream names or CLSID naming doc, xls, ppt, msi, outlook, thumbsdb, hwp or vsd; generic ole never relabels hinted specific formats; streams not decoded"
SIGNATURE = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"
END, FREE, FATSECT, DIFSECT = 0xFFFFFFFE, 0xFFFFFFFF, 0xFFFFFFFD, 0xFFFFFFFC
MSI_CLSID = uuid.UUID("000C1084-0000-0000-C000-000000000046").bytes_le
MSP_CLSID = uuid.UUID("000C1086-0000-0000-C000-000000000046").bytes_le
ENTRIES = 1_000_000


class Malformed(Exception):
    pass


class Compound:
    def __init__(self, data: bytes):
        self.data = data
        major, _, shift, mini_shift = struct.unpack_from("<HHHH", data, 26)
        if major not in (3, 4) or shift not in (9, 12) or mini_shift != 6:
            raise Malformed("Header version or sector shift invalid")
        self.sector = 1 << shift
        (
            self.fat_sectors,
            self.first_dir,
            _,
            self.cutoff,
            self.first_mini,
            self.mini_sectors,
            self.first_difat,
            self.difat_sectors,
        ) = struct.unpack_from("<IIIIIIII", data, 44)
        self.total = (
            len(data) // self.sector - 1
        )  # sector n starts at (n + 1) * sector size; v4 pads the header to 4096
        self.fat = self.read_fat()

    def offset(self, sector: int) -> int:
        if sector >= self.total:
            raise Malformed(f"Sector {sector} outside file")
        return (sector + 1) * self.sector

    def entries(self, sector: int) -> list[int]:
        start = self.offset(sector)
        return list(struct.unpack_from(f"<{self.sector // 4}I", self.data, start))

    def read_fat(self) -> list[int]:
        difat = list(struct.unpack_from("<109I", self.data, 76))
        sector, seen = self.first_difat, 0
        while sector not in (END, FREE) and seen < self.difat_sectors:
            block = self.entries(sector)
            difat.extend(block[:-1])
            sector = block[-1]
            seen += 1
        fat = []
        for index, sector in enumerate(difat):
            if sector in (END, FREE):
                continue
            if index >= self.fat_sectors:
                break
            fat.extend(self.entries(sector))
        if len(fat) < self.total and self.fat_sectors:
            fat.extend([FREE] * (self.total - len(fat)))
        return fat

    def chain(self, start: int, table: list[int], needed: int) -> int:
        """Count sectors in a chain, requiring at least `needed`; loops and overruns fail."""
        count, sector, seen = 0, start, set()
        while sector not in (END, FREE) and count < needed:
            if sector in seen or sector >= len(table):
                raise Malformed("Chain loops or leaves the allocation table")
            seen.add(sector)
            if table is self.fat:
                self.offset(sector)
            count += 1
            sector = table[sector]
        if count < needed:
            raise Malformed("Chain shorter than the stream size")
        return count

    def directory(self) -> list[tuple[str, int, int, int, bytes]]:
        """(name, type, start, size, clsid) for every directory entry, walking the chain."""
        found, sector, seen = [], self.first_dir, set()
        while sector not in (END, FREE):
            if sector in seen or sector >= len(self.fat):
                raise Malformed("Directory chain loops or leaves the FAT")
            seen.add(sector)
            base = self.offset(sector)
            for slot in range(self.sector // 128):
                at = base + slot * 128
                length, kind = struct.unpack_from("<HB", self.data, at + 64)
                if kind == 0:
                    continue
                name = self.data[at : at + max(length - 2, 0)].decode("utf-16-le", "replace")
                start, size = struct.unpack_from("<IQ", self.data, at + 116)
                found.append((name, kind, start, size, self.data[at + 80 : at + 96]))
                if len(found) > ENTRIES:
                    raise Malformed("Directory budget exceeded")
            sector = self.fat[sector]
        if not found or found[0][1] != 5:
            raise Malformed("First directory entry is not the root storage")
        return found


def dispatch(names: set[str], root_clsid: bytes) -> tuple[str, str]:
    lowered = {name.lower() for name in names}
    if root_clsid in (MSI_CLSID, MSP_CLSID):
        return "msi", "Windows Installer root CLSID"
    if "worddocument" in lowered:
        return "doc", "WordDocument stream"
    if "workbook" in lowered or "book" in lowered:
        return "xls", "Workbook stream"
    if "powerpoint document" in lowered:
        return "ppt", "PowerPoint Document stream"
    if "visiodocument" in lowered:
        return "visio", "VisioDocument stream"
    if "fileheader" in lowered and "bodytext" in lowered:
        return "hwp", "Hangul FileHeader and BodyText"
    if "__properties_version1.0" in lowered or any(
        name.startswith("__substg1.0_") for name in lowered
    ):
        return "outlook", "MAPI property streams"
    if "catalog" in lowered and any(name.isdigit() for name in lowered):
        return "thumbsdb", "Thumbnail catalog"
    if any(re.fullmatch(r"\d+_[0-9a-f]{16}", name) for name in lowered):
        return "thumbsdb", "Vista-style thumbnail streams"
    if any(0x3800 <= ord(name[0]) <= 0x4840 for name in names if name):
        return "msi", "Installer-encoded stream names"
    return "ole", "no recognised application streams"


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if not data.startswith(SIGNATURE):
        return None
    if len(data) < 1536:
        return Observation("fail", "Truncated compound file", "ole")
    try:
        compound = Compound(data)
        entries = compound.directory()
        for name, kind, start, size, _ in entries:
            if kind != 2 or size == 0:
                continue
            if size >= compound.cutoff:
                compound.chain(start, compound.fat, -(-size // compound.sector))
        root_size = entries[0][3]
        if root_size:
            compound.chain(entries[0][2], compound.fat, -(-root_size // compound.sector))
        if compound.mini_sectors:
            mini = []
            sector = compound.first_mini
            for _ in range(compound.mini_sectors):
                if sector in (END, FREE):
                    break
                mini.extend(compound.entries(sector))
                sector = compound.fat[sector] if sector < len(compound.fat) else END
            for name, kind, start, size, _ in entries[1:]:
                if kind == 2 and 0 < size < compound.cutoff:
                    compound.chain(start, mini, -(-size // 64))
        if len(data) % compound.sector:
            raise Malformed("File is not a whole number of sectors")
    except Malformed as error:
        return Observation("fail", str(error), "ole")
    except struct.error:
        return Observation("fail", "Truncated structure", "ole")
    names = {name for name, kind, _, _, _ in entries[1:] if kind in (1, 2)}
    kind, reason = dispatch(names, entries[0][4])
    return Observation(
        "pass",
        f"CFB with {len(entries)} directory entries and chains verified; {reason}",
        kind,
        generic=kind == "ole",
    )
