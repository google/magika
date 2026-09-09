# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""One command for rule correctness, coverage and actual-product performance."""

import argparse
import json
import os
import subprocess
from pathlib import Path

from . import corpus, report, runner


def integers(value):
    try:
        result = sorted(set(map(int, value.split(","))))
    except ValueError as error:
        raise argparse.ArgumentTypeError("Expected comma-separated integers") from error
    if not result or any(n < 0 for n in result):
        raise argparse.ArgumentTypeError("Values must be nonnegative")
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset", type=Path)
    parser.add_argument("--model-config", type=Path)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--rules-file", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--phase", choices=("all", "quality", "performance", "render"), default="all"
    )
    parser.add_argument("--counts", type=integers, default=[10, 100, 1000])
    parser.add_argument("--hit-rates", type=integers, default=list(range(0, 101, 5)))
    parser.add_argument("--workers", type=integers, default=[4])
    parser.add_argument("--backends", nargs="+", choices=("cpu", "gpu"), default=["cpu"])
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--seed", type=int, default=20260906)
    parser.add_argument("--timeout", type=float, default=60)
    parser.add_argument("--resume", action="store_true")
    args = parser.parse_args(argv)
    if (
        min(args.counts + args.workers) < 1
        or max(args.hit_rates) > 100
        or args.repeats < 1
        or args.timeout <= 0
    ):
        parser.error(
            "Counts, workers, repeats and timeout must be positive; hit rates must be 0..100"
        )
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    raw_path = output / "results.json"
    if args.phase == "render":
        result = json.loads(raw_path.read_text())
    else:
        if not args.binary or not args.model_config:
            parser.error("--binary and --model-config are required for measurements")
        binary = args.binary.resolve()
        pack = args.rules_file.resolve() if args.rules_file else output / "rules.yar"
        env = dict(os.environ, LC_ALL="C")
        env.setdefault("MAGIKA_RULES_CACHE", str(output / "cache"))
        if not args.rules_file:
            exported = output / "embedded-rules.tmp.yar"
            if exported.exists():
                exported.unlink()
            subprocess.run(
                [str(binary), "--write-default-rules", str(exported)],
                env=env,
                check=True,
                capture_output=True,
                timeout=args.timeout,
            )
            exported.replace(pack)
        identity = dict(
            binary=corpus.file_hash(binary),
            rules=corpus.file_hash(pack),
            config=corpus.file_hash(args.model_config),
        )
        if args.phase in ("all", "quality"):
            if not args.dataset:
                parser.error("--dataset is required for correctness evaluation")
            classes, samples, data_identity = corpus.prepare(args.dataset, output)
            mapping, supported = corpus.class_mapping(
                classes, json.loads(args.model_config.read_text())
            )
            observations, rules = runner.observe(
                binary, pack, samples, mapping, env, timeout=args.timeout
            )
            result = dict(
                schema=1,
                identity=identity,
                corpus=data_identity,
                classes=classes,
                supported=supported,
                samples=observations,
                rules=rules,
            )
            corpus.atomic_json(raw_path, result)
        else:
            result = json.loads(raw_path.read_text())
            if result["identity"] != identity:
                raise ValueError("Saved observations belong to a different binary, rules or model")
        if args.phase in ("all", "performance") and not report.failures(result):
            eligible = [r for r in result["samples"] if r["truth"] in result["supported"]]
            if not eligible:
                raise ValueError(
                    "No supported labeled files are available for performance evaluation"
                )
            result["performance"] = runner.matrix(
                binary,
                pack,
                eligible,
                output / "performance.json",
                env,
                counts=args.counts,
                rates=args.hit_rates,
                workers=args.workers,
                backends=args.backends,
                repeats=args.repeats,
                seed=args.seed,
                timeout=args.timeout,
                resume=args.resume,
            )
    text = report.render(result)
    temporary = output / "report.md.tmp"
    temporary.write_text(text)
    temporary.replace(output / "report.md")
    corpus.atomic_json(raw_path, result)
    print(f"Generated {output / 'report.md'}", flush=True)
    failures = report.failures(result)
    if failures:
        raise SystemExit("Evaluation failed: " + "; ".join(failures))
