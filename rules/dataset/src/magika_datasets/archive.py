# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Package verified public metadata without corpus bytes or credentials."""

import argparse
import gzip
import hashlib
import io
import json
import tarfile
import tempfile
import time
from pathlib import Path

from .acquisition import sha256_file
from .parquet_corpus import check_sources
from .runtime import private_output
from .unattended import read, write

BASE_FILES = (
    "classes.parquet",
    "samples.parquet",
    "repositories.parquet",
    "repository-files.parquet",
    "github-used.parquet",
    "corpus-parquet-receipt.json",
    "repositories-receipt.json",
    "repository-files-receipt.json",
    "github-used-receipt.json",
    "metadata-downloads.json",
)
CANDIDATE_FILES = (
    "classes.parquet",
    "samples.parquet",
    "candidate-index.parquet",
    "taxonomy.parquet",
    "corpus-parquet-receipt.json",
)


def package(metadata: Path, output: Path, candidates: Path | None = None) -> dict:
    """Use an explicit allowlist and atomically publish after full archive readback."""
    receipt = read(metadata / "corpus-parquet-receipt.json")
    check_sources(metadata / "samples.parquet", metadata / "classes.parquet", receipt)
    for file in read(metadata / "metadata-downloads.json")["files"]:
        if sha256_file(metadata / file["name"]) != file["sha256"]:
            raise ValueError(f"Metadata checksum mismatch: {file['name']}")
    inputs = {name: metadata / name for name in BASE_FILES}
    for name in ("validation.parquet", "validation-summary.json"):
        if (metadata / name).exists():
            inputs[name] = metadata / name
    if candidates is not None:
        candidate_receipt = read(candidates / "corpus-parquet-receipt.json")
        check_sources(
            candidates / "samples.parquet", candidates / "classes.parquet", candidate_receipt
        )
        for name, expected in candidate_receipt["files"].items():
            if sha256_file(candidates / name) != expected["sha256"]:
                raise ValueError(f"Candidate metadata checksum mismatch: {name}")
        inputs.update({"candidate-metadata/" + name: candidates / name for name in CANDIDATE_FILES})
        # Canonical taxonomy may evolve independently of the frozen discovery run.
        inputs["candidate-metadata/taxonomy.parquet"] = metadata / "classes.parquet"
        if (candidates / "github-used.parquet").exists():
            inputs["candidate-metadata/github-used.parquet"] = candidates / "github-used.parquet"
    if candidates is not None:
        for name in ("validation.parquet", "validation.json"):
            if (candidates / name).exists():
                inputs["candidate-metadata/" + name] = candidates / name
    members = {
        name: {"sha256": sha256_file(path), "bytes": path.stat().st_size}
        for name, path in sorted(inputs.items())
    }
    manifest = {
        "schema_version": 1,
        "scope": "metadata only; no corpus bytes",
        "accepted_samples": receipt["samples"],
        "candidate_samples": candidate_receipt["samples"] if candidates else 0,
        "candidate_labels": "manual; unresolved and conflicting retained",
        "files": members,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent) as temporary:
        archive = Path(temporary) / "metadata.tar.gz"
        with (
            archive.open("wb") as raw,
            gzip.GzipFile(fileobj=raw, mode="wb", mtime=0, filename="") as zipped,
        ):
            with tarfile.open(fileobj=zipped, mode="w") as tar:
                for name, path in sorted(inputs.items()):
                    info = tarfile.TarInfo(name)
                    info.size, info.mode = path.stat().st_size, 0o644
                    with path.open("rb") as stream:
                        tar.addfile(info, stream)
                data = (json.dumps(manifest, sort_keys=True, indent=2) + "\n").encode()
                info = tarfile.TarInfo("archive-manifest.json")
                info.size, info.mode = len(data), 0o644
                tar.addfile(info, io.BytesIO(data))
        with tarfile.open(archive, "r:gz") as tar:
            for member in tar.getmembers():
                if member.name == "archive-manifest.json":
                    continue
                with tar.extractfile(member) as stream:
                    if (
                        hashlib.file_digest(stream, "sha256").hexdigest()
                        != members[member.name]["sha256"]
                    ):
                        raise ValueError(f"Archive readback mismatch: {member.name}")
        result = {
            **{k: v for k, v in manifest.items() if k != "files"},
            "archive": output.name,
            "bytes": archive.stat().st_size,
            "sha256": sha256_file(archive),
            "verified": True,
            "download_url": None,
        }
        archive.replace(output)
    write(output.with_suffix(output.suffix + ".json"), result)
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--candidates", type=Path)
    parser.add_argument(
        "--wait-for",
        type=Path,
        help="Package once an existing pipeline completes; never launches collection",
    )
    parser.add_argument(
        "--validate-store", type=Path, help="Revalidate completed candidates before archiving"
    )
    args = parser.parse_args(argv)
    output = private_output(args.output)
    if args.wait_for:
        from .pipeline import pipeline_status

        while True:
            status = pipeline_status(args.wait_for)
            if status["phase"] == "complete":
                args.candidates = args.wait_for / "metadata"
                break
            if status["phase"] in {"failed", "interrupted", "not_started"}:
                raise ValueError("Pipeline did not complete; existing archive preserved")
            time.sleep(30)
    if args.validate_store and args.candidates:
        from .validation import validate_dataset

        validate_dataset(
            args.candidates, args.validate_store, args.candidates / "validation.parquet"
        )
    print(json.dumps(package(args.metadata, output, args.candidates), sort_keys=True), flush=True)


if __name__ == "__main__":
    main()
