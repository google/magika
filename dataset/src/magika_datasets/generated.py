# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Samples whose class is defined by how they were made, reproducible from their origin.

Some classes Magika names have no files in the wild to collect: an empty file, bytes with
no structure, characters or words in no language, a symbolic link stored as the path it
points to. Their samples are generated. Each origin records the generator, its version
and an index, and the bytes derive only from those, so anyone can regenerate a sample and
compare its SHA-256 -- which is what makes the origin evidence rather than a claim.
"""

import hashlib
import re

VERSION = 1
ORIGIN = re.compile(
    r"^generated:(?P<kind>[a-z]+):v(?P<version>\d+):(?P<index>\d+):(?P<sha256>[0-9a-f]{64})$"
)
MIN_SIZE, MAX_SIZE = 8, 64 * 1024  # Magika reads content from 8 bytes; keep samples small
LETTERS = b"abcdefghijklmnopqrstuvwxyz"
PATH_CHARACTERS = b"abcdefghijklmnopqrstuvwxyz0123456789_-"
EXTENSIONS = (b"", b"", b".so", b".so.1", b".dylib", b".txt", b".json", b".py", b".h", b".conf")


class Stream:
    """Deterministic bytes: SHA-256 in counter mode over a seed naming the sample."""

    def __init__(self, kind: str, index: int):
        self.seed = f"magika-datasets/generated/{kind}/v{VERSION}/{index}".encode()
        self.counter, self.buffer = 0, b""

    def bytes(self, count: int) -> bytes:
        while len(self.buffer) < count:
            self.buffer += hashlib.sha256(self.seed + self.counter.to_bytes(8, "big")).digest()
            self.counter += 1
        out, self.buffer = self.buffer[:count], self.buffer[count:]
        return out

    def below(self, bound: int) -> int:
        """Uniform integer in [0, bound) by rejection, so no value is favoured."""
        width = (bound - 1).bit_length() + 7 >> 3 or 1
        limit = (1 << 8 * width) // bound * bound
        while True:
            value = int.from_bytes(self.bytes(width), "big")
            if value < limit:
                return value % bound

    def size(self) -> int:
        """Log-uniform between MIN_SIZE and MAX_SIZE, so small and large files both occur."""
        low, high = MIN_SIZE.bit_length(), MAX_SIZE.bit_length()
        bits = low + self.below(high - low)
        return min(MAX_SIZE, max(MIN_SIZE, (1 << bits - 1) + self.below(1 << bits - 1)))


def _empty(stream: Stream) -> bytes:
    return b""


def _randombytes(stream: Stream) -> bytes:
    return stream.bytes(stream.size())


def _randomascii(stream: Stream) -> bytes:
    return bytes(0x20 + stream.below(0x5F) for _ in range(stream.size()))


def _randomtxt(stream: Stream) -> bytes:
    """Words of random letters with spaces, punctuation and line breaks: text in no language."""
    target, out, line = stream.size(), bytearray(), 0
    while len(out) < target:
        word = bytes(LETTERS[stream.below(26)] for _ in range(1 + stream.below(12)))
        if stream.below(8) == 0:
            word = word[:1].upper() + word[1:]
        out += word
        line += len(word)
        mark = stream.below(20)
        if mark == 0:
            out += b"."
        elif mark == 1:
            out += b","
        if line > 60 + stream.below(40):
            out += b"\n"
            line = 0
        else:
            out += b" "
            line += 1
    return bytes(out[:target])


def _symlinktext(stream: Stream) -> bytes:
    """A link target as Git stores a symbolic link: the path alone, no trailing newline."""
    parts = []
    lead = stream.below(4)
    prefix = b"/" if lead == 0 else b"../" * stream.below(4) if lead == 1 else b""
    for _ in range(1 + stream.below(5)):
        parts.append(
            bytes(
                PATH_CHARACTERS[stream.below(len(PATH_CHARACTERS))]
                for _ in range(1 + stream.below(14))
            )
        )
    return prefix + b"/".join(parts) + EXTENSIONS[stream.below(len(EXTENSIONS))]


GENERATORS = {
    "empty": _empty,
    "randombytes": _randombytes,
    "randomascii": _randomascii,
    "randomtxt": _randomtxt,
    "symlinktext": _symlinktext,
}
DISTINCT = {"empty": 1}
"""Classes that cannot hold more distinct samples than this: every empty file is the same file."""


def generate(kind: str, index: int) -> bytes:
    if kind not in GENERATORS:
        raise ValueError(f"No generator for {kind}")
    return GENERATORS[kind](Stream(kind, index))


def origin(kind: str, index: int) -> tuple[str, bytes]:
    data = generate(kind, index)
    return f"generated:{kind}:v{VERSION}:{index}:{hashlib.sha256(data).hexdigest()}", data


def identity(value: str) -> str | None:
    """The class a generated origin names, only if regenerating it reproduces its SHA-256."""
    match = ORIGIN.match(value)
    if not match or int(match["version"]) != VERSION or match["kind"] not in GENERATORS:
        return None
    data = generate(match["kind"], int(match["index"]))
    return match["kind"] if hashlib.sha256(data).hexdigest() == match["sha256"] else None
