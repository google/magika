# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Safe XML parsing: entity declarations and external resources are refused.

A bare DOCTYPE (QGIS projects, SVG and XHTML from older tools) is allowed; the
dangerous constructs are entity expansion and external fetches, which stay off.
"""

from xml.etree.ElementTree import Element, ParseError

from defusedxml.common import DefusedXmlException
from defusedxml.ElementTree import fromstring


class Unsafe(Exception):
    """Document declares entities; it is not processed."""


class Malformed(Exception):
    """Document is not well-formed XML."""


class Unsupported(Exception):
    """Document declares an encoding expat cannot decode; well-formedness is undecided."""


def parse(data: bytes) -> Element:
    try:
        return fromstring(data, forbid_dtd=False, forbid_entities=True, forbid_external=True)
    except DefusedXmlException as error:
        raise Unsafe("Entity-declaring XML not processed") from error
    except ParseError as error:
        raise Malformed("Malformed XML") from error
    except ValueError as error:  # expat: "multi-byte encodings are not supported"
        raise Unsupported("Unsupported XML encoding declaration") from error
