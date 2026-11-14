# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Package verified public metadata without corpus bytes or credentials."""

import argparse
import hashlib
import json
import shutil
import tempfile
import zipfile
from pathlib import Path

from .acquisition import sha256_file
from .parquet_corpus import check_sources
from .runtime import private_output, read, write

# Each derived table and the receipt that names its checksum. samples and classes are bound
# by corpus-parquet-receipt.json instead, through check_sources.
RECEIPTS = {
    "github-used.parquet": "github-used-receipt.json",
    "repositories.parquet": "repositories-receipt.json",
    "repository-files.parquet": "repository-files-receipt.json",
    "repository-licenses.parquet": "repository-licenses-receipt.json",
}
TABLES = ("classes.parquet", "samples.parquet", *RECEIPTS)
COLLECTIONS = ("sembiance",)
"""Further collections on the corpus taxonomy, each a directory of the same three files."""
DOWNLOADS = "metadata-downloads.json"
EPOCH = (1980, 1, 1, 0, 0, 0)


def package(metadata: Path, output: Path, download_url: str | None = None) -> dict:
    """Use an explicit allowlist and atomically publish after full archive readback."""
    receipt = read(metadata / "corpus-parquet-receipt.json")
    check_sources(metadata / "samples.parquet", metadata / "classes.parquet", receipt)
    for table, name in RECEIPTS.items():
        if sha256_file(metadata / table) != read(metadata / name)["parquet_sha256"]:
            raise ValueError(f"Metadata checksum mismatch: {table} is not what {name} records")
    inputs = {name: metadata / name for name in TABLES}
    inputs["corpus-parquet-receipt.json"] = metadata / "corpus-parquet-receipt.json"
    inputs.update({name: metadata / name for name in RECEIPTS.values()})
    for collection in COLLECTIONS:
        directory = metadata / collection
        if not (directory / "corpus-parquet-receipt.json").exists():
            continue
        check_sources(
            directory / "samples.parquet",
            directory / "classes.parquet",
            read(directory / "corpus-parquet-receipt.json"),
        )
        for name in ("classes.parquet", "samples.parquet", "corpus-parquet-receipt.json"):
            inputs[f"{collection}/{name}"] = directory / name
    members = {
        name: {"sha256": sha256_file(path), "bytes": path.stat().st_size}
        for name, path in sorted(inputs.items())
    }
    manifest = {
        "schema_version": 1,
        "scope": "metadata only; no corpus bytes",
        "samples": receipt["samples"],
        "files": members,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent) as temporary:
        archive = Path(temporary) / "metadata.zip"
        # Zip opens natively on every platform. Parquet is already compressed, so members are
        # stored, and a fixed timestamp keeps the same inputs producing the same bytes.
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_STORED) as bundle:
            for name, path in sorted(inputs.items()):
                info = zipfile.ZipInfo(name, date_time=EPOCH)
                info.external_attr = 0o644 << 16
                with path.open("rb") as source, bundle.open(info, "w", force_zip64=True) as sink:
                    shutil.copyfileobj(source, sink, 1024 * 1024)
            info = zipfile.ZipInfo("archive-manifest.json", date_time=EPOCH)
            bundle.writestr(info, json.dumps(manifest, sort_keys=True, indent=2) + "\n")
        with zipfile.ZipFile(archive) as bundle:
            for name in bundle.namelist():
                if name == "archive-manifest.json":
                    continue
                with bundle.open(name) as stream:
                    if hashlib.file_digest(stream, "sha256").hexdigest() != members[name]["sha256"]:
                        raise ValueError(f"Archive readback mismatch: {name}")
        result = {
            **{k: v for k, v in manifest.items() if k != "files"},
            "archive": output.name,
            "bytes": archive.stat().st_size,
            "sha256": sha256_file(archive),
            "verified": True,
            "download_url": download_url,
        }
        archive.replace(output)
    write(output.with_suffix(output.suffix + ".json"), result)
    # The download list is derived from what was just verified and packaged, so it cannot
    # drift from the tables the way a hand-kept list did.
    write(
        metadata / DOWNLOADS,
        {
            "schema_version": 1,
            "download_url": download_url,
            "scope": "Public metadata only. Place these files in this directory."
            if download_url
            else "Public metadata only. Hosting location will be added before release. "
            "Place these files in this directory.",
            "files": [
                {"name": name, **members[name]}
                for name in sorted(members)
                if name.endswith(".parquet")
            ],
        },
    )
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path, required=True)
    # Passed in rather than hand-written afterwards, so the archive, its sidecar and the
    # download list always name the same hosting location.
    parser.add_argument("--download-url", help="where the published archive is hosted")
    args = parser.parse_args(argv)
    result = package(args.metadata, private_output(args.output), args.download_url)
    print(json.dumps(result, sort_keys=True), flush=True)


if __name__ == "__main__":
    main()
