# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Build, hydrate and inspect reproducible datasets."""

import argparse
import importlib
import sys

COMMANDS = {
    "hydrate": ("hydrate", "materialize verified whole-file Parquet shards"),
    "validate": ("validation", "revalidate samples into label-status groups"),
    "taxonomy": ("taxonomy", "apply reviewed class additions and corrections"),
    "refill": ("refill", "fill short classes from GitHub, VirusTotal or generators"),
    "prune": ("prune", "drop samples the corpus may not redistribute"),
    "sembiance": ("sembiance", "import Sembiance's file-format samples as a second collection"),
    "fingerprint": ("fingerprints", "fingerprint samples for near-duplicate exclusion"),
    "select": ("selection", "draw a balanced evaluation subset"),
    "sources": ("sources", "export the GitHub repositories the corpus draws from"),
    "licenses": ("licenses", "resolve and publish repository licences"),
    "stats": ("stats", "read-only corpus statistics"),
    "receipt": ("receipt", "rebind the published metadata to its digests"),
    "archive": ("archive", "package public metadata for distribution"),
    "benchmark-config": ("benchmark.config", "write the detector benchmark configuration"),
    "benchmark": ("benchmark.run", "benchmark Magika against other detectors on a snapshot"),
}


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    parser = argparse.ArgumentParser(
        description=__doc__,
        epilog="\n".join(f"  {name:16} {help}" for name, (_, help) in COMMANDS.items()),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("command", choices=list(COMMANDS))
    if not argv or argv[0] in ("-h", "--help"):
        parser.print_help()
        return
    module, _ = COMMANDS[parser.parse_args(argv[:1]).command]
    return importlib.import_module(f".{module}", __package__).main(argv[1:])
