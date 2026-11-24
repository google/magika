# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""LaTeX sources: control sequences, balanced groups and matched environments outside comments and verbatim."""

import re

from ..contract import Observation

FAMILY = "text"
FORMAT_IDS = ("latex",)
SCOPE = "Hinted sources only: % comments skipped, \\verb delimiters and verbatim-style environments skipped, { } groups balanced, every \\begin{env} matched by \\end{env} in order, at least three control sequences; mismatches are inconclusive because catcode and macro tricks make static balance checks non-proof; macros not expanded"
REQUIRES_HINT = True
CONTEXT_REQUIRED = True
VERBATIM = {b"verbatim", b"verbatim*", b"lstlisting", b"minted", b"Verbatim", b"comment", b"alltt"}
CONTROL = re.compile(rb"\\([A-Za-z@]+\*?|.)", re.S)
ENV = re.compile(rb"\s*\{([^}\s]+)\}")


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if "latex" not in hints:
        return None
    position, depth, sequences, stack = 0, 0, 0, []
    length = len(data)
    while position < length:
        byte = data[position]
        if byte == 0x25:  # comment to end of line
            end = data.find(b"\n", position)
            position = length if end < 0 else end + 1
        elif byte == 0x7B:
            depth += 1
            position += 1
        elif byte == 0x7D:
            depth -= 1
            if depth < 0:
                return Observation(
                    "inconclusive", f"Unbalanced closing brace at byte {position}", "latex"
                )
            position += 1
        elif byte == 0x5C:
            match = CONTROL.match(data, position)
            if not match:
                return Observation("inconclusive", "Dangling backslash at EOF", "latex")
            name = match.group(1)
            position = match.end()
            sequences += name.isalpha() or name.endswith(b"*")
            if name in (b"verb", b"verb*"):
                if position >= length:
                    return Observation("inconclusive", "\\verb without a delimiter", "latex")
                end = data.find(data[position : position + 1], position + 1)
                if end < 0:
                    return Observation("inconclusive", "Unterminated \\verb", "latex")
                position = end + 1
            elif name in (b"begin", b"end"):
                env = ENV.match(data, position)
                if not env:
                    if name == b"end":
                        continue  # plain TeX end-of-job primitive
                    return Observation(
                        "inconclusive", "\\begin without an environment name", "latex"
                    )
                position = env.end()
                environment = env.group(1)
                if b"#" in environment or b"\\" in environment:
                    continue  # macro definitions build environment names at run time
                if name == b"begin":
                    if environment in VERBATIM:
                        end = data.find(b"\\end{" + environment + b"}", position)
                        if end < 0:
                            return Observation(
                                "inconclusive",
                                f"Unterminated {environment.decode()} environment",
                                "latex",
                            )
                        position = end + len(environment) + 6
                    else:
                        stack.append(environment)
                elif not stack or stack[-1] != environment:
                    return Observation(
                        "inconclusive",
                        f"\\end{{{environment.decode()}}} does not match the open environment",
                        "latex",
                    )
                else:
                    stack.pop()
        else:
            position += 1
    if depth:
        return Observation("inconclusive", "Unbalanced braces at EOF", "latex")
    if stack:
        return Observation("inconclusive", f"Unclosed environment {stack[-1].decode()}", "latex")
    if sequences < 3:
        return Observation("inconclusive", "Fewer than three control sequences", "latex")
    return Observation(
        "pass", f"{sequences} control sequences; groups and environments balanced", "latex"
    )
