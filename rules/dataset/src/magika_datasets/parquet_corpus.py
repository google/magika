# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Whole-file Parquet shards with exact metadata and byte verification."""

import hashlib
import json
import shutil
import tempfile
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq

from .acquisition import object_path, sha256_file


def metadata_line(row):
    return (
        json.dumps(
            {k: v.hex() if isinstance(v, bytes) else v for k, v in row.items()},
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n"
    ).encode()


def metadata_rows(path):
    for batch in pq.ParquetFile(path).iter_batches(batch_size=64):
        yield from batch.to_pylist()


def check_sources(samples, classes, receipt):
    for name, path in [("samples.parquet", samples), ("classes.parquet", classes)]:
        if sha256_file(path) != receipt["files"][name]["sha256"]:
            raise ValueError("Public Parquet metadata differs from its receipt")
    if pq.ParquetFile(samples).metadata.num_rows != receipt["samples"]:
        raise ValueError("Public sample count mismatch")
    if pq.ParquetFile(classes).metadata.num_rows != receipt["classes"]:
        raise ValueError("Public class count mismatch")


def verify(output, samples, classes):
    manifest = json.loads((output / "manifest.json").read_text())
    if sha256_file(output / "classes.parquet") != sha256_file(classes):
        raise ValueError("Parquet corpus class metadata mismatch")
    expected = iter(metadata_rows(samples))
    count, size, metadata_hash = 0, 0, hashlib.sha256()
    listed = [r["path"] for r in manifest["shards"]]
    if len(listed) != len(set(listed)) or set(listed) != {
        str(p.relative_to(output)) for p in (output / "shards").glob("*.parquet")
    }:
        raise ValueError("Parquet shard inventory mismatch")
    for shard in manifest["shards"]:
        path = (output / shard["path"]).resolve()
        if (
            not path.is_relative_to((output / "shards").resolve())
            or sha256_file(path) != shard["sha256"]
        ):
            raise ValueError("Parquet shard path or hash mismatch")
        seen = 0
        for batch in pq.ParquetFile(path).iter_batches(batch_size=32):
            for row in batch.to_pylist():
                content = row.pop("content")
                prior = next(expected, None)
                if prior != row:
                    raise ValueError("Parquet corpus sample metadata mismatch")
                if len(content) != row["size"] or hashlib.sha256(content).digest() != row["sha256"]:
                    raise ValueError("Parquet corpus whole-file bytes mismatch")
                count += 1
                seen += 1
                size += len(content)
                metadata_hash.update(metadata_line(row))
        if seen != shard["samples"]:
            raise ValueError("Parquet shard sample count mismatch")
    if (
        next(expected, None) is not None
        or count != manifest["samples"]
        or size != manifest["original_bytes"]
    ):
        raise ValueError("Parquet corpus is incomplete")
    if metadata_hash.hexdigest() != manifest["metadata_rows_sha256"]:
        raise ValueError("Parquet corpus metadata digest mismatch")
    return {
        "verified": True,
        "samples": count,
        "original_bytes": size,
        "shards": len(listed),
        "classes": pq.ParquetFile(classes).metadata.num_rows,
        "metadata_rows_sha256": metadata_hash.hexdigest(),
        "whole_file_sha256_checked": count,
        "tensorflow_required": False,
    }


def pack(samples, classes, receipt, store, output, *, shard_bytes=64 * 1024**2, progress=None):
    if output.exists():
        raise ValueError("Choose a new Parquet snapshot output")
    if not 1024 <= shard_bytes <= 256 * 1024**2:
        raise ValueError("Parquet shard budget must be 1 KiB..256 MiB")
    check_sources(samples, classes, receipt)
    output.parent.mkdir(parents=True, exist_ok=True)
    schema = pq.ParquetFile(samples).schema_arrow.append(pa.field("content", pa.binary()))
    with tempfile.TemporaryDirectory(
        prefix=output.name + ".building-", dir=output.parent
    ) as directory:
        root = Path(directory)
        (root / "shards").mkdir()
        shutil.copyfile(classes, root / "classes.parquet")
        writer, writer_path = None, None
        manifest = {
            "schema_version": 1,
            "format": "parquet-whole-file-v1",
            "samples": 0,
            "original_bytes": 0,
            "compression": "zstd:3",
            "shards": [],
            "source_metadata_receipt": receipt,
            "pyarrow_version": pa.__version__,
        }
        metadata_hash = hashlib.sha256()
        buffer, buffered_bytes, shard_size, shard_count = [], 0, 0, 0

        def flush():
            nonlocal buffered_bytes
            if buffer:
                writer.write_table(pa.Table.from_pylist(buffer, schema=schema))
                buffer.clear()
                buffered_bytes = 0

        def close_shard():
            nonlocal writer
            if writer is not None:
                flush()
                writer.close()
                writer = None
                manifest["shards"].append(
                    {
                        "path": str(writer_path.relative_to(root)),
                        "sha256": sha256_file(writer_path),
                        "bytes": writer_path.stat().st_size,
                        "samples": shard_count,
                    }
                )
                if progress:
                    progress(
                        {"shards_written": len(manifest["shards"]), "samples": manifest["samples"]}
                    )

        try:
            for row in metadata_rows(samples):
                if type(row["size"]) is not int or not 0 <= row["size"] <= 8 * 1024**2:
                    raise ValueError("Existing sample exceeds 8-MiB historical replay budget")
                path = object_path(store, row["sha256"].hex())
                if path.stat().st_size != row["size"]:
                    raise ValueError("Stored corpus object size mismatch")
                content = path.read_bytes()
                if hashlib.sha256(content).digest() != row["sha256"]:
                    raise ValueError("Stored corpus object SHA-256 mismatch")
                if writer is not None and shard_size + len(content) > shard_bytes:
                    close_shard()
                if writer is None:
                    writer_path = root / "shards" / f"part-{len(manifest['shards']):05d}.parquet"
                    writer = pq.ParquetWriter(
                        writer_path,
                        schema,
                        compression="zstd",
                        compression_level=3,
                        use_dictionary=["format_id", "class_ordinal", "hard_case"],
                    )
                    shard_size, shard_count = 0, 0
                metadata_hash.update(metadata_line(row))
                buffer.append({**row, "content": content})
                buffered_bytes += len(content)
                shard_size += len(content)
                shard_count += 1
                manifest["samples"] += 1
                manifest["original_bytes"] += len(content)
                if len(buffer) >= 64 or buffered_bytes >= 8 * 1024**2:
                    flush()
            close_shard()
        finally:
            if writer is not None:
                writer.close()
        manifest["metadata_rows_sha256"] = metadata_hash.hexdigest()
        if manifest["samples"] != receipt["samples"]:
            raise ValueError("Not all public samples were packed")
        (root / "manifest.json").write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
        verification = verify(root, samples, classes)
        (root / "verification.json").write_text(
            json.dumps(verification, sort_keys=True, indent=2) + "\n"
        )
        root.rename(output)
    return verification
