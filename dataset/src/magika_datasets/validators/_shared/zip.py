# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Bounded ZIP walk: central directory, member bounds, CRCs and safe names."""

import io
import zipfile
from dataclasses import dataclass
from xml.etree.ElementTree import Element

from . import xml

MEMBERS = 4096
EXPANDED = 64 * 1024 * 1024
MEMBER = 16 * 1024 * 1024
EOCD = b"PK\x05\x06"


class Malformed(Exception):
    """Archive structure, CRC or member naming is broken."""


class Budget(Exception):
    """Member count or expanded bytes exceed the budget."""


class Unsupported(Exception):
    """Encryption or a compression method the standard library cannot expand."""


class Failure(Exception):
    """A structural failure attributed to a specific format."""

    def __init__(self, format_id: str, detail: str):
        super().__init__(detail)
        self.format_id = format_id
        self.detail = detail


@dataclass(frozen=True)
class Package:
    names: list[str]
    infos: list[zipfile.ZipInfo]
    contents: dict[str, bytes]


def unsafe(name: str) -> bool:
    return name.startswith("/") or "\\" in name or ".." in name.split("/")


def walk(data: bytes) -> Package:
    if not data.startswith(b"PK\x03\x04"):
        raise Malformed("ZIP local header absent")
    try:
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            infos = archive.infolist()
            names = [info.filename for info in infos]
            if (
                len(infos) > MEMBERS
                or sum(info.file_size for info in infos) > EXPANDED
                or any(info.file_size > MEMBER for info in infos)
            ):
                raise Budget("Package member or expanded-byte budget exceeded")
            if len(set(names)) != len(names) or any(unsafe(name) for name in names):
                raise Malformed("Duplicate or unsafe member names")
            if any(info.flag_bits & 1 for info in infos):
                raise Unsupported("Encrypted ZIP members")
            end = len(data) - 22 - len(archive.comment)
            if data[end : end + 4] != EOCD:
                raise Malformed("Trailing bytes after end of central directory")
            contents = {}
            for info in infos:
                payload = archive.read(info)
                if len(payload) != info.file_size:
                    raise Malformed("Expanded member size mismatch")
                contents[info.filename] = payload
    except (zipfile.BadZipFile, ValueError, KeyError, OSError, EOFError) as error:
        raise Malformed("Malformed ZIP structure or CRC") from error
    except (NotImplementedError, RuntimeError) as error:
        raise Unsupported("Unsupported ZIP compression or encryption") from error
    return Package(names, infos, contents)


def member_xml(package: Package, name: str, format_id: str) -> Element:
    """Parse one member; malformed XML fails the named format, unsafe XML propagates."""
    try:
        return xml.parse(package.contents[name])
    except xml.Malformed as error:
        raise Failure(format_id, f"Malformed XML in {name}") from error
