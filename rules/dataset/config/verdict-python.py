# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Small process adapters for locally installed Python identification tools."""

import argparse
import json
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('tool', choices=['magika', 'puremagic'])
    parser.add_argument('source', type=Path)
    parser.add_argument('path', type=Path)
    args = parser.parse_args()
    sys.path.insert(0, str(args.source.resolve()))
    if args.tool == 'magika':
        from magika import Magika
        result = Magika().identify_path(args.path).asdict()
    else:
        import puremagic
        try:
            result = [r._asdict() for r in puremagic.magic_file(str(args.path))]
        except puremagic.PureError as exc:
            result = {'matches': [], 'message': str(exc)}
    print(json.dumps(result, default=lambda x: x.hex() if isinstance(x, bytes) else str(x)))


if __name__ == '__main__':
    main()
