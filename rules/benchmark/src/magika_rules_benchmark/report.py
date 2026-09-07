# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Compute evidence and render tables from recorded results, without running tools."""

import math
import statistics
from collections import Counter, defaultdict

from tabulate import tabulate


def ratio(numerator, denominator):
    return numerator / denominator if denominator else None


def confusion(tp, fp, fn, tn=0):
    return dict(
        tp=tp,
        fp=fp,
        fn=fn,
        tn=tn,
        precision=ratio(tp, tp + fp),
        recall=ratio(tp, tp + fn),
        f1=ratio(2 * tp, 2 * tp + fp + fn),
        fp_rate=ratio(fp, fp + tn),
        fn_rate=ratio(fn, tp + fn),
    )


def auc(buckets):
    positives = sum(v[0] for v in buckets.values())
    negatives = sum(v[1] for v in buckets.values())
    if not positives or not negatives:
        return None
    tp, area = 0, 0.0
    for score, (positive, negative) in sorted(buckets.items(), reverse=True):
        if not math.isfinite(score):
            raise ValueError("Nonfinite prediction score")
        area += negative * (2 * tp + positive)
        tp += positive
    return area / (2 * positives * negatives)


def classification(rows, labels):
    rows = [r for r in rows if r["truth"] is not None and not r["ambiguous"]]
    result = {}
    labels = set(labels) | {
        row[field]
        for row in rows
        for field in ("rule_prediction", "ml_prediction", "hybrid_prediction")
        if row[field] is not None
    }
    for mode in ("rules", "ml", "hybrid"):
        field = "rule_prediction" if mode == "rules" else f"{mode}_prediction"
        hits = sum(r[field] is not None for r in rows)
        tp = sum(r[field] == r["truth"] for r in rows)
        buckets = defaultdict(lambda: [0, 0])
        for row in rows:
            for label in labels:
                score = (
                    (1.0 if mode == "rules" else row[f"{mode}_score"])
                    if row[field] == label
                    else 0.0
                )
                if not math.isfinite(score) or not 0 <= score <= 1:
                    raise ValueError("Invalid classification score")
                buckets[score][0 if row["truth"] == label else 1] += 1
        result[mode] = confusion(
            tp, hits - tp, len(rows) - tp, len(rows) * (len(labels) - 1) - (hits - tp)
        ) | dict(
            accuracy=ratio(tp, len(rows)), coverage=ratio(hits, len(rows)), roc_auc=auc(buckets)
        )
    return result


def metrics(result):
    rows, rules = result["samples"], result["rules"]
    supported = set(result["supported"])
    known = [r for r in rows if r["truth"] is not None and not r["ambiguous"]]
    per_rule = {}
    for name, rule in rules.items():
        positives = sum(r["truth"] == rule["format_id"] for r in known)
        targets = [r for r in known if name in r["raw_matches"]]
        tp = sum(r["truth"] == rule["format_id"] for r in targets)
        fp, fn = len(targets) - tp, positives - tp
        ambiguous = sum(r["ambiguous"] and name in r["raw_matches"] for r in rows)
        unadjudicated = sum(
            r["truth"] is None and not r["ambiguous"] and name in r["raw_matches"] for r in rows
        )
        enabled = rule.get("enforced", rule.get("enabled", False))
        status = (
            "not_working"
            if fp or ambiguous or unadjudicated or not tp or not enabled
            else "partial"
            if fn
            else "full"
        )
        per_rule[name] = confusion(tp, fp, fn, len(known) - positives - fp) | dict(
            format_id=rule["format_id"],
            status=status,
            ambiguous_matches=ambiguous,
            unadjudicated_matches=unadjudicated,
            conflicts=dict(Counter(r["truth"] for r in targets if r["truth"] != rule["format_id"])),
        )
    primary = [r for r in known if r["truth"] in supported]
    additional = [r for r in known if r["truth"] not in supported]
    return dict(
        classification=classification(primary, supported),
        additional=classification(additional, {r["truth"] for r in additional}),
        supported_samples=len(primary),
        additional_samples=len(additional),
        unadjudicated=sum(r["truth"] is None and not r["ambiguous"] for r in rows),
        ambiguous=sum(r["ambiguous"] for r in rows),
        reference_errors=sum(bool(r["reference_error"]) for r in rows),
        reference_mismatches=sum(r["reference_mismatch"] for r in rows),
        conflicts=sum(r["conflict"] for r in rows),
        required_abstention_violations=sum(
            r["ambiguous"] and r["rule_prediction"] is not None for r in rows
        ),
        corrected=sum(
            r["ml_prediction"] != r["truth"] and r["hybrid_prediction"] == r["truth"]
            for r in primary
        ),
        introduced=sum(
            r["ml_prediction"] == r["truth"] and r["hybrid_prediction"] != r["truth"]
            for r in primary
        ),
        per_rule=per_rule,
    )


def percent(value):
    return "—" if value is None else f"{100 * value:.1f}%"


def table(headers, rows):
    def cell(value):
        if value is None:
            return "—"
        if isinstance(value, float):
            return f"{value:.1f}"
        return str(value).replace("|", "\\|").replace("\n", " ")

    return tabulate(
        [[cell(v) for v in row] for row in rows],
        headers=headers,
        tablefmt="github",
        disable_numparse=True,
    )


def performance_rows(performance):
    values = {}
    for cell in performance.get("measurements", []):
        trials = cell["trials"]
        elapsed = statistics.median(t["seconds"] for t in trials)
        rss = [t["peak_rss_bytes"] for t in trials if t["peak_rss_bytes"] is not None]
        values[
            cell["backend"], cell["files"], cell["hit_percent"], cell["workers"], cell["mode"]
        ] = dict(
            ms=1000 * elapsed, fps=cell["files"] / elapsed, rss=max(rss) / 2**20 if rss else None
        )
    return values


def render(result):
    totals = metrics(result)
    result["metrics"] = totals
    rows = []
    for mode, value in totals["classification"].items():
        rows.append(
            [
                mode,
                value["tp"],
                value["fp"],
                value["fn"],
                *[
                    percent(value[k])
                    for k in ("precision", "recall", "f1", "accuracy", "coverage", "roc_auc")
                ],
            ]
        )
    lines = [
        "# Magika rules evaluation",
        "",
        "## Accuracy, coverage and performance",
        "",
        f"{totals['supported_samples']:,} labeled files in the model's supported classes. "
        f"Hybrid corrects {totals['corrected']} ML errors and introduces {totals['introduced']}.",
        "",
        table(
            [
                "Mode",
                "TP",
                "FP",
                "FN",
                "Precision",
                "Recall",
                "F1",
                "Accuracy",
                "Decision coverage",
                "ROC AUC",
            ],
            rows,
        ),
        "",
    ]
    by_type = defaultdict(list)
    for value in totals["per_rule"].values():
        by_type[value["format_id"]].append(value)
    if any(len(values) > 1 for values in by_type.values()):
        raise ValueError("Expected one reconciled terminal rule per canonical file type")
    inventory = Counter(
        by_type[name][0]["status"] if name in by_type else "none" for name in result["supported"]
    )
    lines += [
        table(
            ["Full", "Partial", "Not working", "None"],
            [[inventory[k] for k in ("full", "partial", "not_working", "none")]],
        ),
        "",
    ]
    performance = result.get("performance") or {}
    speed = performance_rows(performance)
    coverage = totals["classification"]["rules"]["coverage"] or 0
    headline = []
    for backend in sorted({key[0] for key in speed}):
        for count in (10, 100, 1000):
            rates = [
                k[2] for k in speed if k[0] == backend and k[1] == count and k[3:] == (4, "hybrid")
            ]
            if not rates:
                continue
            rate = min(rates, key=lambda value: abs(value - 100 * coverage))
            ml = speed[backend, count, rate, 4, "ml"]
            hybrid = speed[backend, count, rate, 4, "hybrid"]
            headline.append(
                [
                    backend,
                    count,
                    f"{rate}%",
                    ml["fps"],
                    hybrid["fps"],
                    percent(hybrid["fps"] / ml["fps"] - 1),
                    ml["ms"],
                    hybrid["ms"],
                    hybrid["rss"],
                ]
            )
    lines += [
        table(
            [
                "Backend",
                "Files",
                "Rule hits",
                "ML files/s",
                "Hybrid files/s",
                "Gain",
                "ML ms",
                "Hybrid ms",
                "Peak RSS MiB",
            ],
            headline,
        )
        if headline
        else "Performance was not run for the headline cases.",
        "",
        "Speed uses four workers and the measured controlled mix nearest corpus rule coverage. "
        "Times are median whole-process elapsed time over shuffled identical inputs, including startup, "
        "disk reads, classification, output and shutdown. RSS is the maximum measured process peak. "
        "File data and rule caches are warmed; Parquet decoding is excluded.",
        "",
    ]
    lines += [
        "## Definitions and checks",
        "",
        "TP: correct class decision. FP: assigning another class's file to this class. "
        "FN: missing a file of this class, including abstention. Full: zero observed FP/FN. "
        "Partial: zero FP and some FN. Not working: disabled, observed FP, ambiguous matches or no "
        "positive evidence. None: no rule. False negatives can fall through to ML; false positives block rules.",
        "",
        "ROC AUC uses the returned label's confidence and zero for other labels; rule matches score 1. "
        "It is not AUC over the full model probability vector. Missing denominators are shown as —. "
        "Rule scores are deterministic decisions, not calibrated probabilities. "
        "Dataset annotations provide truth; these results do not establish a universal zero-error guarantee.",
        "",
        table(
            ["Check", "Count"],
            [
                [key.replace("_", " "), totals[key]]
                for key in (
                    "additional_samples",
                    "unadjudicated",
                    "ambiguous",
                    "conflicts",
                    "reference_errors",
                    "reference_mismatches",
                    "required_abstention_violations",
                )
            ],
        ),
        "",
    ]
    if performance:
        host = performance["host"]
        lines += [
            table(
                ["Hardware", "Value"],
                [
                    [k, host[k]]
                    for k in (
                        "platform",
                        "architecture",
                        "cpu",
                        "logical_cpus",
                        "physical_cores",
                        "performance_cores",
                        "efficiency_cores",
                    )
                    if k in host
                ]
                + ([["RAM GiB", host["ram_bytes"] / 2**30]] if host.get("ram_bytes") else []),
            ),
            "",
        ]
    performance = result.get("performance", {})
    reads = performance.get("read_baseline", {})
    if reads.get("trials"):
        lines += [
            table(
                ["I/O baseline", "Value"],
                [
                    [
                        "Read MiB/s (median)",
                        statistics.median(t["mib_per_second"] for t in reads["trials"]),
                    ]
                ],
            ),
            "",
            reads["scope"],
            "",
        ]
    storage = performance.get("host", {}).get("storage")
    if isinstance(storage, dict):
        lines += [
            table(
                ["Storage", "Value"],
                [
                    [k, storage[k]]
                    for k in ("FilesystemType", "BusProtocol", "SolidState")
                    if k in storage
                ],
            ),
            "",
        ]
    for title, names in [
        ("Appendix A — supported file types", result["supported"]),
        (
            "Appendix B — additional file types with rules",
            sorted(set(by_type) - set(result["supported"])),
        ),
    ]:
        lines += [f"## {title}", ""]
        families = defaultdict(list)
        for name in names:
            info = result["classes"].get(name, {})
            family = " / ".join(info.get("categories", [])) or "Uncategorized"
            value = by_type.get(name, [None])[0]
            conflicts = []
            if value:
                for other, count in sorted(value["conflicts"].items()):
                    extensions = ", ".join(result["classes"].get(other, {}).get("extensions", []))
                    conflicts.append(f"{other} ({extensions or 'no standard extension'}): {count}")
            families[family].append(
                [
                    name,
                    value["status"] if value else "none",
                    value["fp"] if value else None,
                    value["fn"] if value else None,
                    "; ".join(conflicts) or "—",
                ]
            )
        for family, entries in sorted(families.items()):
            lines += [
                f"### {family}",
                "",
                table(["File type", "Status", "FP", "FN", "FP conflicts"], entries),
                "",
            ]
    lines += ["## Appendix C — performance detail", ""]
    for backend, count, workers in sorted({(key[0], key[1], key[3]) for key in speed}):
        entries = []
        for rate in sorted(
            {k[2] for k in speed if (k[0], k[1], k[3]) == (backend, count, workers)}
        ):
            ml, hybrid = [speed[backend, count, rate, workers, mode] for mode in ("ml", "hybrid")]
            entries.append(
                [
                    f"{rate}%",
                    ml["fps"],
                    hybrid["fps"],
                    percent(hybrid["fps"] / ml["fps"] - 1),
                    ml["ms"],
                    hybrid["ms"],
                    ml["rss"],
                    hybrid["rss"],
                ]
            )
        lines += [
            f"### {backend}: {count} files, {workers} workers",
            "",
            table(
                [
                    "Rule hits",
                    "ML files/s",
                    "Hybrid files/s",
                    "Gain",
                    "ML ms",
                    "Hybrid ms",
                    "ML RSS MiB",
                    "Hybrid RSS MiB",
                ],
                entries,
            ),
            "",
        ]
    return "\n".join(lines)


def failures(result):
    """Disabled candidates may fail; enforced terminals and engine parity may not."""
    values = metrics(result)
    errors = [
        f"{name}: {values[name]}"
        for name in ("reference_errors", "reference_mismatches", "required_abstention_violations")
        if values[name]
    ]
    for name, rule in result["rules"].items():
        measured = values["per_rule"][name]
        if rule.get("enforced", rule.get("enabled", False)) and (
            measured["fp"] or measured["ambiguous_matches"] or measured["unadjudicated_matches"]
        ):
            errors.append(
                f"Enforced rule {name} produced a false positive, ambiguous or unadjudicated match"
            )
    return errors
