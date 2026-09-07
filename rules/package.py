#!/usr/bin/env python3
# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Stage source crates or bundle an existing executable with its exact embedded rules."""

import argparse
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def stage_source(output):
    output.mkdir(parents=True, exist_ok=False)
    for name, paths in {
        "lib": ("Cargo.toml", "Cargo.lock", "LICENSE", "README.md", "build.rs", "src"),
        "tract-runtime": ("Cargo.toml", "Cargo.lock", "LICENSE", "README.md", "src", "models"),
    }.items():
        destination = output / name
        destination.mkdir()
        for relative in paths:
            source = ROOT.parent / "rust" / name / relative
            if source.is_dir():
                shutil.copytree(source, destination / relative)
            else:
                shutil.copyfile(source, destination / relative)
    shutil.copytree(ROOT / "rulesets", output / "lib/rulesets")
    shutil.copyfile(ROOT / "LICENSES", output / "lib/RULES-LICENSES")
    return output / "lib/Cargo.toml"


def bundle(binary, library, license_file, output):
    if output.exists():
        raise FileExistsError(output)
    if library.name.endswith(".dylib"):
        library_name = "libhs.dylib"
    elif library.name.endswith(".dll"):
        library_name = "hs.dll"
    elif ".so" in library.name:
        library_name = "libhs.so.5"
    else:
        raise ValueError("Expected a Vectorscan dylib, DLL or shared object")
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="magika-package-", dir=output.parent) as temporary:
        work = Path(temporary)
        root = work / "bundle"
        for directory in ("lib", "rules", "licenses"):
            (root / directory).mkdir(parents=True)
        executable = root / ("magika.exe" if binary.suffix == ".exe" else "magika")
        shutil.copy2(binary, executable)
        shutil.copy2(library, root / "lib" / library_name)
        shutil.copyfile(license_file, root / "licenses/vectorscan.txt")
        shutil.copyfile(ROOT.parent / "LICENSE", root / "licenses/magika.txt")
        shutil.copyfile(ROOT / "LICENSES", root / "licenses/rules.txt")
        shutil.copyfile(ROOT / "README.md", root / "README.md")
        source = root / "rules/promoted.yar"
        env = dict(os.environ)
        env.pop("MAGIKA_VECTORSCAN_LIBRARY", None)
        for arguments in (("--write-default-rules", source), ("--compile-rules", source)):
            subprocess.run(
                [str(executable), *map(str, arguments)],
                env=env,
                check=True,
                capture_output=True,
                timeout=120,
            )
        # A nonempty probe verifies native discovery even when all bundled rules are disabled.
        probe = work / "probe.yar"
        probe.write_text(
            'rule probe { meta: label = "unknown" enforced = true class = "full" '
            "fp_rate = 0 fn_rate = 0 condition: uint8(0) == 0 }"
        )
        subprocess.run(
            [str(executable), "--compile-rules", str(probe)],
            env=env,
            check=True,
            capture_output=True,
            timeout=120,
        )
        archive = work / "bundle.tar.gz"
        with tarfile.open(archive, "w:gz") as stream:
            for path in sorted(root.rglob("*")):
                stream.add(path, arcname=str(path.relative_to(root)), recursive=False)
        os.link(archive, output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="mode", required=True)
    source = commands.add_parser(
        "source", help="Stage self-contained library and local runtime sources"
    )
    source.add_argument("--output", type=Path, required=True)
    distribution = commands.add_parser("bundle", help="Bundle an already built native CLI")
    for name in ("binary", "library", "license", "output"):
        distribution.add_argument(f"--{name}", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "source":
        print(stage_source(args.output.resolve()))
    else:
        bundle(
            args.binary.resolve(),
            args.library.resolve(),
            args.license.resolve(),
            args.output.resolve(),
        )


if __name__ == "__main__":
    main()
