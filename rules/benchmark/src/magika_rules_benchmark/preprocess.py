# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Python reimplementation of the native preprocessors, so YARA-X can act as the oracle.

The native engine scans two streams: stream A is the file prefix, stream B a 64-byte facts
header followed by the 4096-byte `zip_names` view and the 16 KiB `zip_first_entry` view,
derived from the first 4 KiB block, the input size and (for archives only) a 16 KiB tail
window, extended back to a central directory of at most 256 KiB. YARA-X cannot run those
preprocessors, but it evaluates the same conditions when every fact is an integer global
and every view a bytes global. This module is a stdlib-only, byte-exact port of the
contracts documented at the top of `rust/lib/src/rules/preprocess/{mod,zip,pe}.rs`: the
facts table, the tail window (`read_tail`), the zip analysis, the PE analysis and
`prepare`, which derives the facts and the view from the held blocks. The benchmark
runner and the test fixtures call `define_globals` once per compiler and `set_globals`
per input with the output of `prepare` (or of `facts_for`, which reads the blocks too).

The native engine scans stream B only when a preprocessor produced something
(`is_active`): a facts rule never matches an input no preprocessor touched, and the
native compiler rejects a rule that the all-zero header and the empty view would
satisfy. YARA-X has no such gate, and the facts of a family that did not fire are
supplied as zeros, so it would match such a rule where native never scans; the two
agree exactly on every rule the compiler accepts.

The view reaches YARA-X as `bytes`, which YARA-X stores verbatim, so a `contains` or
`startswith` literal with non-ASCII bytes matches exactly the bytes the native engine
sees. Passing it as `str` instead (decoded as latin-1) would not: YARA-X re-encodes a
`str` global to UTF-8, so a non-ASCII name would then diverge from native.
"""

import os
import zlib
from pathlib import Path

# The facts header, `(name, offset, width)` big-endian unsigned fields; offsets are stable
# and part of the native cache identity.
FACTS = (
    ("original_size", 0, 8),
    ("prefix_size", 8, 8),
    ("zip_valid", 16, 1),
    ("zip_flags", 17, 1),
    ("zip_entries", 18, 2),
    ("zip_names_entries", 20, 2),
    ("zip_names_len", 22, 2),
    ("zip_comment_len", 24, 2),
    ("zip_cd_size", 26, 4),
    ("pe_valid", 32, 1),
    ("pe_flags", 33, 1),
    ("pe_machine", 34, 2),
    ("pe_characteristics", 36, 2),
    ("pe_subsystem", 38, 2),
    ("pe_dll_characteristics", 40, 2),
    ("pe_sections", 42, 2),
    ("pe_magic", 44, 2),
    ("pe_clr", 46, 1),
    ("pe_signed", 47, 1),
    ("pe_overlay", 48, 8),
    ("pe_is_dll", 56, 1),
    ("pe_is_executable_image", 57, 1),
    ("zip_first_entry_len", 30, 2),
)
FACT_NAMES = tuple(name for name, _, _ in FACTS)
ZIP_FACT_NAMES = tuple(name for name in FACT_NAMES if name.startswith("zip_"))
PE_FACT_NAMES = tuple(name for name in FACT_NAMES if name.startswith("pe_"))

FACTS_BYTES = 64
ZIP_NAMES_BYTES = 4096
ZIP_FIRST_ENTRY_BYTES = 16 * 1024
# The views: `(name, stream B offset, size)`, contiguous after the facts header.
VIEWS = (
    ("zip_names", FACTS_BYTES, ZIP_NAMES_BYTES),
    ("zip_first_entry", FACTS_BYTES + ZIP_NAMES_BYTES, ZIP_FIRST_ENTRY_BYTES),
)
VIEW_NAMES = tuple(name for name, _, _ in VIEWS)
# The bytes of stream B after the facts header: every view, in stream order.
VIEWS_BYTES = ZIP_NAMES_BYTES + ZIP_FIRST_ENTRY_BYTES

# The first block every input pays for, and the trailing window read for archives.
PREFIX_BYTES = 4096
ZIP_TAIL_BYTES = 16 * 1024

# --- zip ---------------------------------------------------------------------------------

# Largest central directory read on its own when the tail window does not hold it.
ZIP_DIRECTORY_MAX_BYTES = 256 * 1024
ZIP_FLAG_ZIP64 = 1 << 0
ZIP_FLAG_MULTIDISK = 1 << 1
ZIP_FLAG_CD_NOT_HELD = 1 << 2
ZIP_FLAG_MALFORMED = 1 << 3
ZIP_FLAG_NAMES_TRUNCATED = 1 << 4
ZIP_FLAG_PREPENDED = 1 << 5
ZIP_FLAG_FIRST_ENTRY_FILLED = 1 << 6
ZIP_FLAG_FIRST_ENTRY_UNDECODABLE = 1 << 7
# Any of these bits clears `zip_valid`; a walked zip64 archive, names truncated and prepended
# data do not.
_ZIP_INVALID = ZIP_FLAG_MULTIDISK | ZIP_FLAG_CD_NOT_HELD | ZIP_FLAG_MALFORMED

_LOCAL_SIGNATURE = b"PK\x03\x04"
_CENTRAL_SIGNATURE = b"PK\x01\x02"
_EOCD_SIGNATURE = b"PK\x05\x06"
_ZIP64_LOCATOR_SIGNATURE = b"PK\x06\x07"
_ZIP64_EOCD_SIGNATURE = b"PK\x06\x06"
_LOCAL_HEADER_LEN = 30
_CENTRAL_HEADER_LEN = 46
_EOCD_LEN = 22
_ZIP64_LOCATOR_LEN = 20
_ZIP64_EOCD_LEN = 56
_MIMETYPE_MAX_LEN = 128
# The view sanitizer: 0x00 and 0x0A become 0x01 inside every line's payload.
_SANITIZE = bytes.maketrans(b"\x00\n", b"\x01\x01")

# --- pe ----------------------------------------------------------------------------------

PE_MAX_SECTIONS = 96
PE_FLAG_SECTIONS_NOT_HELD = 1 << 0
PE_FLAG_CERTIFICATE_OUT_OF_FILE = 1 << 1
PE_FLAG_UNKNOWN_MAGIC = 1 << 2
PE_FLAG_SECTION_BEYOND_FILE = 1 << 3
PE32_MAGIC = 0x10B
PE32PLUS_MAGIC = 0x20B

_E_LFANEW_AT = 0x3C
_PE_SIGNATURE = b"PE\0\0"
_COFF_HEADER_LEN = 20
_SECTION_HEADER_LEN = 40
# The fixed optional header fields by magic; the data directories follow them.
_PE_FIXED_LEN = {PE32_MAGIC: 96, PE32PLUS_MAGIC: 112}
_DIRECTORY_ENTRIES = 16
_DIRECTORY_CERTIFICATE_TABLE = 4
_DIRECTORY_CLR_RUNTIME_HEADER = 14
_CHARACTERISTIC_EXECUTABLE_IMAGE = 0x0002
_CHARACTERISTIC_DLL = 0x2000


def _u16(data, at):
    """The little-endian u16 at `at`, or None when it does not lie entirely inside `data`."""
    if at < 0 or at + 2 > len(data):
        return None
    return int.from_bytes(data[at : at + 2], "little")


def _u32(data, at):
    """The little-endian u32 at `at`; see `_u16`."""
    if at < 0 or at + 4 > len(data):
        return None
    return int.from_bytes(data[at : at + 4], "little")


def _u64(data, at):
    """The little-endian u64 at `at`; see `_u16`."""
    if at < 0 or at + 8 > len(data):
        return None
    return int.from_bytes(data[at : at + 8], "little")


def wants_tail(prefix):
    """Whether the prefix opens a zip archive, the only inputs whose tail is read."""
    return prefix.startswith(_LOCAL_SIGNATURE) or prefix.startswith(_EOCD_SIGNATURE)


def tail_window(size, prefix):
    """The `(start, end)` window read for an input of `size` bytes opening with `prefix`.

    Mirrors the native `read_tail`: only a prefix that `wants_tail` and does not already
    hold the whole input costs a read, of the last `min(size, ZIP_TAIL_BYTES)` bytes.
    """
    if not wants_tail(prefix) or size <= len(prefix):
        return None
    return max(0, size - ZIP_TAIL_BYTES), size


def read_tail(stream, size, first):
    """The tail window of the open `stream`, or None when `tail_window` reads nothing.

    `first` is the block already read from the start of the stream; the stream is left
    positioned after the window when one was read.
    """
    window = tail_window(size, first)
    if window is None:
        return None
    start, end = window
    stream.seek(start)
    tail = stream.read(end - start)
    extension = directory_start(tail, size)
    if extension is not None:
        stream.seek(extension)
        tail = stream.read(start - extension) + tail
    return tail


def directory_start(tail, size):
    """Where to start reading so that the central directory is held, as the native
    `directory_start`: `tail` (ending at `size`) holds the end record of a single-disk,
    non-zip64 archive whose directory starts before `tail` and spans at most
    `ZIP_DIRECTORY_MAX_BYTES`; None otherwise.
    """
    eocd = _find_eocd(tail)
    tail_start = size - len(tail)
    if eocd is None or tail_start < 0:
        return None
    _, _, cd_size32, _, flags = _decode_eocd(tail, eocd)
    resolved = _zip64_directory(tail, eocd, tail_start)
    if resolved is not None:
        _, cd_size, _, cd_end_abs = resolved
    elif flags:
        return None
    else:
        cd_size, cd_end_abs = cd_size32, tail_start + eocd
    if cd_size > ZIP_DIRECTORY_MAX_BYTES:
        return None
    start = cd_end_abs - cd_size
    return start if 0 <= start < tail_start else None


def _stored_mimetype(first):
    """The payload of a stored, short `mimetype` first entry held entirely in `first`."""
    if not first.startswith(_LOCAL_SIGNATURE):
        return None
    method, name_len, length, extra_len = (
        _u16(first, 8),
        _u16(first, 26),
        _u32(first, 22),
        _u16(first, 28),
    )
    if None in (method, name_len, length, extra_len):
        return None
    if method != 0 or name_len != 8 or not 1 <= length <= _MIMETYPE_MAX_LEN:
        return None
    name_end = _LOCAL_HEADER_LEN + 8
    if first[_LOCAL_HEADER_LEN:name_end] != b"mimetype":
        return None
    data_start = name_end + extra_len
    if data_start + length > len(first):
        return None
    return first[data_start : data_start + length]


def _find_eocd(tail):
    """Position in `tail` of the end-of-central-directory record whose comment reaches the end.

    The last 22-byte window and every earlier one is tried, walking backwards.
    """
    if len(tail) < _EOCD_LEN:
        return None
    # A candidate starts at most `len(tail) - 22` in: its signature ends before `end`.
    end = len(tail) - _EOCD_LEN + len(_EOCD_SIGNATURE)
    while True:
        at = tail.rfind(_EOCD_SIGNATURE, 0, end)
        if at < 0:
            return None
        if _u16(tail, at + 20) == len(tail) - at - _EOCD_LEN:
            return at
        end = at + len(_EOCD_SIGNATURE) - 1


def _entry_name(directory, at):
    """The name of the central header at `at` and the whole entry's length, when held."""
    if at + _CENTRAL_HEADER_LEN > len(directory):
        return None
    header = directory[at : at + _CENTRAL_HEADER_LEN]
    if not header.startswith(_CENTRAL_SIGNATURE):
        return None
    name_len = _u16(header, 28)
    trailer = _u16(header, 30) + _u16(header, 32)
    name_start = at + _CENTRAL_HEADER_LEN
    name_end = name_start + name_len
    # The extra field and comment must fit as well, or the next header cannot be found.
    entry_end = name_end + trailer
    if entry_end > len(directory):
        return None
    return directory[name_start:name_end], entry_end - at


class _ViewWriter:
    """Appends `\\n`-led sanitized lines to the view and terminates the sequence once."""

    def __init__(self):
        self.out = bytearray()

    def fits(self, prefix, text):
        # The leading `\n`, the prefix and the text; the terminator needs one more byte.
        return len(self.out) + 1 + len(prefix) + len(text) < ZIP_NAMES_BYTES

    def line(self, prefix, text):
        self.out += b"\n" + prefix + text.translate(_SANITIZE)

    def finish(self):
        if self.out:
            self.out += b"\n"
        return bytes(self.out)


def _zip64_directory(tail, eocd, tail_start):
    """The `(entries, cd_size, cd_offset, cd_end_abs)` of a zip64 archive whose zip64 end record
    is held in `tail`, or None when there is no locator or its record is not held. `cd_end_abs`
    is where the directory ends, i.e. where the zip64 end record begins.
    """
    locator = eocd - _ZIP64_LOCATOR_LEN
    if locator < 0 or not tail[locator:].startswith(_ZIP64_LOCATOR_SIGNATURE):
        return None
    record_abs = _u64(tail, locator + 8)
    if record_abs is None:
        return None
    at = record_abs - tail_start
    if at < 0 or at + _ZIP64_EOCD_LEN > len(tail):
        return None
    record = tail[at : at + _ZIP64_EOCD_LEN]
    if not record.startswith(_ZIP64_EOCD_SIGNATURE):
        return None
    entries, cd_size, cd_offset = _u64(record, 32), _u64(record, 40), _u64(record, 48)
    if None in (entries, cd_size, cd_offset):
        return None
    return entries, cd_size, cd_offset, record_abs


def _walk_directory(first, size, tail, eocd, view, facts):
    """Decodes the EOCD at `eocd` in `tail`, places the directory and walks its names."""
    tail_start = size - len(tail)
    if tail_start < 0:
        return
    entries32, comment_len, cd_size32, cd_offset32, flags = _decode_eocd(tail, eocd)
    facts["zip_entries"] = entries32
    facts["zip_comment_len"] = comment_len
    if flags & ZIP_FLAG_MULTIDISK:
        facts["zip_flags"] = flags
        return
    eocd_abs = tail_start + eocd
    # The directory ends at the EOCD, or, for a zip64 archive, at the zip64 end record its
    # locator points to. A claimed but unresolvable zip64 archive keeps its flag and is invalid.
    resolved = _zip64_directory(tail, eocd, tail_start)
    if resolved is not None:
        flags |= ZIP_FLAG_ZIP64
        entries, cd_size, cd_offset, cd_end_abs = resolved
    elif flags & ZIP_FLAG_ZIP64:
        facts["zip_flags"] = flags
        return
    else:
        entries, cd_size, cd_offset, cd_end_abs = entries32, cd_size32, cd_offset32, eocd_abs
    facts["zip_cd_size"] = min(cd_size, 0xFFFF_FFFF)
    facts["zip_entries"] = min(entries, 0xFFFF)
    if cd_offset + cd_size < cd_end_abs:
        flags |= ZIP_FLAG_PREPENDED
    cd_abs = cd_end_abs - cd_size
    if cd_abs < 0:
        facts["zip_flags"] = flags | ZIP_FLAG_MALFORMED
        return
    # The directory ends at `cd_end_abs`, so it is inside `tail` iff it starts there; `first`
    # starts at 0, so it is inside `first` iff it ends there.
    if cd_abs >= tail_start:
        start, end = cd_abs - tail_start, cd_end_abs - tail_start
        if end > len(tail) or start > end:
            facts["zip_flags"] = flags | ZIP_FLAG_MALFORMED
            return
        directory = tail[start:end]
    elif cd_end_abs <= len(first):
        directory = first[cd_abs:cd_end_abs]
    else:
        facts["zip_flags"] = flags | ZIP_FLAG_CD_NOT_HELD
        return
    # Top-level names first, then nested ones: both passes walk the same headers. A hostile
    # entry count is bounded by the held directory, which `_entry_name` refuses to leave.
    for nested in (False, True):
        at, stopped = 0, False
        for _ in range(entries):
            entry = _entry_name(directory, at)
            if entry is None:
                flags |= ZIP_FLAG_MALFORMED
                stopped = True
                break
            name, length = entry
            at += length
            if (b"/" in name) != nested:
                continue
            if not view.fits(b"", name):
                flags |= ZIP_FLAG_NAMES_TRUNCATED
                stopped = True
                break
            view.line(b"", name)
            facts["zip_names_entries"] += 1
        if stopped:
            break
    facts["zip_flags"] = flags
    facts["zip_valid"] = int(flags & _ZIP_INVALID == 0)


def _decode_eocd(tail, eocd):
    """`(entries, comment_len, cd_size, cd_offset, flags)` of the end record at `eocd`,
    `flags` holding the zip64 and multi-disk bits of the native step 3.
    """
    record = tail[eocd : eocd + _EOCD_LEN]
    disk, cd_disk = _u16(record, 4), _u16(record, 6)
    entries_disk, entries = _u16(record, 8), _u16(record, 10)
    cd_size, cd_offset = _u32(record, 12), _u32(record, 16)
    locator = eocd >= _ZIP64_LOCATOR_LEN and tail[eocd - _ZIP64_LOCATOR_LEN :].startswith(
        _ZIP64_LOCATOR_SIGNATURE
    )
    flags = 0
    if (
        0xFFFF in (disk, cd_disk, entries_disk, entries)
        or 0xFFFF_FFFF in (cd_size, cd_offset)
        or locator
    ):
        flags |= ZIP_FLAG_ZIP64
    if disk != 0 or cd_disk != 0 or entries_disk != entries:
        flags |= ZIP_FLAG_MULTIDISK
    return entries, _u16(record, 20), cd_size, cd_offset, flags


def first_entry(first):
    """The unpadded `zip_first_entry` view and its flag bits, as the native step 8.

    The name (sanitized) and `\\n`, then the data of a stored or deflated first entry
    held in `first`: copied, or inflated as raw deflate, up to the view's end.
    """
    first = bytes(first)
    if not first.startswith(_LOCAL_SIGNATURE):
        return b"", 0
    flags, method, compressed = _u16(first, 6), _u16(first, 8), _u32(first, 18)
    name_len, extra_len = _u16(first, 26), _u16(first, 28)
    if None in (flags, method, compressed, name_len, extra_len):
        return b"", 0
    if flags & 0b1001 or method not in (0, 8) or name_len + 1 >= ZIP_FIRST_ENTRY_BYTES:
        return b"", 0
    name_end = _LOCAL_HEADER_LEN + name_len
    data_start = name_end + extra_len
    if name_end > len(first) or data_start + compressed > len(first):
        return b"", 0
    head = first[_LOCAL_HEADER_LEN:name_end].translate(_SANITIZE) + b"\n"
    data = first[data_start : data_start + compressed]
    space = ZIP_FIRST_ENTRY_BYTES - len(head)
    if method == 8:
        inflater = zlib.decompressobj(-15)
        try:
            out = inflater.decompress(data, space)
        except zlib.error:
            return head, ZIP_FLAG_FIRST_ENTRY_UNDECODABLE
        complete = inflater.eof
    else:
        out, complete = data[:space], True
    if len(out) == space:
        return head + out, ZIP_FLAG_FIRST_ENTRY_FILLED
    if not complete:
        return head, ZIP_FLAG_FIRST_ENTRY_UNDECODABLE
    return head + out, 0


def zip_analysis(first, size, tail):
    """The seven `zip_*` facts and the unpadded `zip_names` view of an input of `size`
    bytes held as `first` and `tail`: the view is exactly the `zip_names_len` bytes written.
    """
    first, tail = bytes(first), bytes(tail)
    facts = dict.fromkeys(ZIP_FACT_NAMES, 0)
    view = _ViewWriter()
    mimetype = _stored_mimetype(first)
    if mimetype is not None:
        # The line always fits: it is at most 1 + 9 + 128 bytes into an empty view.
        view.line(b"mimetype=", mimetype)
    eocd = _find_eocd(tail)
    if eocd is not None:
        _walk_directory(first, size, tail, eocd, view, facts)
    names = view.finish()
    facts["zip_names_len"] = len(names)
    return facts, names


def _locate_pe(first):
    """Position of the COFF header, when `first` holds `MZ`, `e_lfanew`, `PE\\0\\0` and COFF."""
    if not first.startswith(b"MZ"):
        return None
    e_lfanew = _u32(first, _E_LFANEW_AT)
    if e_lfanew is None:
        return None
    headers_end = e_lfanew + len(_PE_SIGNATURE) + _COFF_HEADER_LEN
    if e_lfanew < 4 or headers_end > len(first):
        return None
    if first[e_lfanew : e_lfanew + len(_PE_SIGNATURE)] != _PE_SIGNATURE:
        return None
    return e_lfanew + len(_PE_SIGNATURE)


def _read_directories(first, size, opt, soh, fixed, facts):
    """The certificate table and CLR runtime header entries, each only when reachable."""
    count = _u32(first, opt + fixed - 4) or 0

    def entry(index):
        if index >= min(count, _DIRECTORY_ENTRIES) or fixed + 8 * (index + 1) > soh:
            return None
        at = opt + fixed + 8 * index
        rva, length = _u32(first, at), _u32(first, at + 4)
        if rva is None or length is None:
            return None
        return rva, length

    certificate = entry(_DIRECTORY_CERTIFICATE_TABLE)
    if certificate is not None:
        offset, length = certificate
        if length != 0:
            if offset + length > size:
                facts["pe_flags"] |= PE_FLAG_CERTIFICATE_OUT_OF_FILE
            else:
                # Offset zero is the DOS header, not a certificate: both halves count.
                facts["pe_signed"] = int(offset != 0)
    clr = entry(_DIRECTORY_CLR_RUNTIME_HEADER)
    if clr is not None:
        rva, length = clr
        facts["pe_clr"] = int(rva != 0 and length != 0)


def pe_facts(first, size):
    """The thirteen `pe_*` facts of an input of `size` bytes whose first block is `first`."""
    first = bytes(first)
    facts = dict.fromkeys(PE_FACT_NAMES, 0)
    coff = _locate_pe(first)
    if coff is None:
        return facts
    facts["pe_machine"] = _u16(first, coff) or 0
    facts["pe_sections"] = sections = _u16(first, coff + 2) or 0
    soh = _u16(first, coff + 16) or 0
    facts["pe_characteristics"] = characteristics = _u16(first, coff + 18) or 0
    facts["pe_is_dll"] = int(characteristics & _CHARACTERISTIC_DLL != 0)
    facts["pe_is_executable_image"] = int(characteristics & _CHARACTERISTIC_EXECUTABLE_IMAGE != 0)
    opt = coff + _COFF_HEADER_LEN
    facts["pe_magic"] = magic = _u16(first, opt) or 0
    facts["pe_subsystem"] = _u16(first, opt + 68) or 0
    facts["pe_dll_characteristics"] = _u16(first, opt + 70) or 0
    fixed = _PE_FIXED_LEN.get(magic)
    if fixed is None:
        facts["pe_flags"] |= PE_FLAG_UNKNOWN_MAGIC
    else:
        _read_directories(first, size, opt, soh, fixed, facts)
    table = opt + soh
    table_end = table + _SECTION_HEADER_LEN * sections
    # The headers are data, not overlay: the raw extent starts where the table ends and
    # only grows past it with the sections' raw data.
    raw_end = table_end
    if table_end > len(first):
        facts["pe_flags"] |= PE_FLAG_SECTIONS_NOT_HELD
    else:
        for index in range(sections):
            header = table + _SECTION_HEADER_LEN * index
            extent = (_u32(first, header + 20) or 0) + (_u32(first, header + 16) or 0)
            if extent > size:
                facts["pe_flags"] |= PE_FLAG_SECTION_BEYOND_FILE
            else:
                raw_end = max(raw_end, extent)
    valid = (
        fixed is not None
        and soh >= fixed
        and 1 <= sections <= PE_MAX_SECTIONS
        and not facts["pe_flags"] & PE_FLAG_SECTIONS_NOT_HELD
        and table_end <= size
    )
    facts["pe_valid"] = int(valid)
    if valid:
        facts["pe_overlay"] = max(size - raw_end, 0)
    return facts


def prepare(first, size, tail):
    """Every fact and the zero-padded `zip_names` view of one input from its held blocks,
    as the native `Synthetic::prepare`: `(facts, view)`.

    `first` is the first block, `size` the input length and `tail` the window `read_tail`
    returned; None when the caller read none, in which case the prefix stands in when it
    holds the whole input and nothing beyond the first block is known otherwise.
    """
    first = bytes(first)
    facts = dict.fromkeys(FACT_NAMES, 0)
    facts["original_size"] = size
    facts["prefix_size"] = len(first)
    views = b""
    if wants_tail(first):
        if tail is None:
            tail = first if size == len(first) else b""
        zip_values, names = zip_analysis(first, size, tail)
        entry, entry_flags = first_entry(first)
        zip_values["zip_first_entry_len"] = len(entry)
        zip_values["zip_flags"] |= entry_flags
        facts.update(zip_values)
        views = names.ljust(ZIP_NAMES_BYTES, b"\0") + entry
    elif first.startswith(b"MZ"):
        facts.update(pe_facts(first, size))
    return facts, views.ljust(VIEWS_BYTES, b"\0")


def facts_for(data_or_path):
    """`prepare` for a file path or for in-memory bytes.

    Reads the first `PREFIX_BYTES` and, when `tail_window` applies, the tail window; any
    other input is not parsed at all beyond its opening signature.
    """
    if isinstance(data_or_path, (str, os.PathLike)):
        path = Path(data_or_path)
        size = path.stat().st_size
        with path.open("rb") as stream:
            first = stream.read(PREFIX_BYTES)
            tail = read_tail(stream, size, first)
        return prepare(first, size, tail)
    data = bytes(data_or_path)
    first = data[:PREFIX_BYTES]
    window = tail_window(len(data), first)
    tail = None
    if window is not None:
        tail = data[window[0] : window[1]]
        extension = directory_start(tail, len(data))
        if extension is not None:
            tail = data[extension:]
    return prepare(first, len(data), tail)


def is_active(facts, view):
    """Whether the native engine scans stream B for this input: a preprocessor produced
    some fact beyond the size header, or a nonempty view (one opens with `\\n`).

    The size facts alone never activate stream B, which stream A already carries.
    """
    return any(facts[name] for name in ZIP_FACT_NAMES + PE_FACT_NAMES) or bool(
        view[:1].strip(b"\0")
    )


def define_globals(compiler):
    """Declares every fact as an integer global and every view as a bytes global."""
    for name in FACT_NAMES:
        compiler.define_global(name, 0)
    for name in VIEW_NAMES:
        compiler.define_global(name, b"")


def set_globals(scanner, facts, views):
    """Sets the globals of one input from `prepare` output on a scanner built with them.

    `views` is the stream B bytes after the facts header, each view at its offset.
    """
    for name in FACT_NAMES:
        scanner.set_global(name, facts[name])
    for name, offset, size in VIEWS:
        start = offset - FACTS_BYTES
        scanner.set_global(name, bytes(views[start : start + size]))
