# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import os
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]


def stage(path):
    subprocess.run(
        [sys.executable, str(ROOT / "package.py"), "source", "--output", str(path)],
        check=True,
        capture_output=True,
    )
    return path / "lib"


def test_source_stage_contains_canonical_inputs_and_no_research(tmp_path):
    library = stage(tmp_path / "source")
    assert (library / "README.md").read_bytes() == (ROOT.parent / "rust/lib/README.md").read_bytes()
    for bucket in ("full", "partial", "notworking"):
        assert (library / "rulesets" / bucket / "formats.yar").read_bytes() == (
            ROOT / "rulesets" / bucket / "formats.yar"
        ).read_bytes()
    assert (library / "RULES-LICENSES").read_bytes() == (ROOT / "LICENSES").read_bytes()
    assert not (library / "src/rules/promoted.yar").exists()
    assert not any(
        path.name in ("tmp", "datasets", "reports", ".venv") for path in library.rglob("*")
    )
    existing = subprocess.run(
        [sys.executable, str(ROOT / "package.py"), "source", "--output", str(library.parent)],
        capture_output=True,
    )
    assert existing.returncode != 0


@pytest.mark.packaging
def test_checkout_build_ignores_stale_packaged_rule_copy(tmp_path):
    library = stage(tmp_path / "repo/rust")
    canonical = tmp_path / "repo/rules/rulesets"
    shutil.copytree(ROOT / "rulesets", canonical)
    (library / "rulesets/full/formats.yar").write_text("this is a stale invalid packaged copy")
    result = subprocess.run(
        [
            "cargo",
            "check",
            "--locked",
            "--manifest-path",
            str(library / "Cargo.toml"),
            "--features",
            "yara-rules",
        ],
        cwd=library,
        env=dict(os.environ, CARGO_TARGET_DIR=str(ROOT.parent / "rust/target")),
        capture_output=True,
        timeout=300,
    )
    assert result.returncode == 0, result.stderr.decode(errors="replace")


@pytest.mark.packaging
def test_staged_crate_builds_without_checkout_rule_paths(tmp_path):
    library = stage(tmp_path / "source")
    env = dict(
        os.environ,
        CARGO_BUILD_JOBS="1",
        CARGO_TARGET_DIR=os.environ.get("CARGO_TARGET_DIR", str(ROOT.parent / "rust/target")),
    )
    for flags in ([], ["--features", "yara-rules"]):
        subprocess.run(
            ["cargo", "check", "--locked", "--manifest-path", str(library / "Cargo.toml"), *flags],
            cwd=library,
            env=env,
            check=True,
            capture_output=True,
            timeout=300,
        )


@pytest.mark.native
def test_bundle_exports_exact_embedded_source_and_compiles(tmp_path):
    binary = os.environ["MAGIKA_TEST_BINARY"]
    exported = tmp_path / "expected.yar"
    subprocess.run(
        [binary, "--write-default-rules", str(exported)], check=True, capture_output=True
    )
    output = tmp_path / "bundle.tar.gz"
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "package.py"),
            "bundle",
            "--binary",
            binary,
            "--library",
            os.environ["MAGIKA_VECTORSCAN_LIBRARY"],
            "--license",
            str(ROOT.parent / "LICENSE"),
            "--output",
            str(output),
        ],
        check=True,
        capture_output=True,
        timeout=120,
    )
    with tarfile.open(output) as archive:
        assert archive.extractfile("rules/promoted.yar").read() == exported.read_bytes()
        assert archive.getmember("rules/promoted.hsdb").size > 0
        assert "README.md" in archive.getnames()
