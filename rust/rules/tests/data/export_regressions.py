# Copyright 2026 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Record every input and expectation of #1447's test_rule_regressions.py as replayable cases.

The PR's tests run unchanged, with the YARA-X scanner replaced by a recorder: `==` records an
exact label set, `in` records a label that must be absent. Inputs derived from a repository
file are stored as that file plus a truncation and byte patches; other inputs are stored as
run-length segments of their prefix. Only the first 4096 bytes and the size matter to a scan.
"""

import itertools
import json
import sys
import types
from pathlib import Path

ROOT = Path(sys.argv[1])  # a tree holding #1447's tests_data and rules/benchmark/tests
OUT = Path(sys.argv[2])
PREFIX = 4096
SOURCE = "google/magika#1447 b771621d rules/benchmark/tests/test_rule_regressions.py"

# Compares #1447's two engines, which this crate replaces, or asserts which rule matched,
# which this crate does not report.
NATIVE_ENGINE = {
    "test_native_engine_agrees_on_adversarial_and_positive_corpus",
    "test_epub_encrypted_mimetype_is_not_accepted_by_the_prefix_rule",
}
# Ported natively in Rust: it walks the repository's own tests_data.
NATIVE_RUST = {"test_all_public_fixture_incomplete_prefixes_abstain"}

# --- a minimal pytest ------------------------------------------------------------------------
pytest = types.ModuleType("pytest")


class Mark:
    def parametrize(self, names, values, ids=None):
        def wrap(fn):
            fn.__dict__.setdefault("_params", []).insert(0, (names, list(values)))
            return fn
        return wrap

    def __getattr__(self, name):
        return lambda fn: fn


pytest.mark = Mark()
pytest.fixture = lambda *a, **k: (a[0] if a and callable(a[0]) else (lambda fn: fn))
pytest.raises = None
sys.modules["pytest"] = pytest
for name in ("yara_x", "magika_rules_benchmark", "magika_rules_benchmark.preprocess",
             "magika_rules_benchmark.corpus", "magika_rules_benchmark.runner"):
    module = types.ModuleType(name)
    module.preprocess = module
    module.file_hash = module.archive = module.pe32 = module.pe32plus = module.stored = None
    module.observe = None
    module.PE32_MAGIC, module.PE32PLUS_MAGIC = 0x10B, 0x20B
    sys.modules[name] = module
# The archive and executable builders of the facts tests are plain Python.
sys.path.insert(0, str(ROOT / "rules/benchmark/tests"))

conftest = {}
exec(compile((ROOT / "rules/benchmark/tests/conftest.py").read_text(), "conftest.py", "exec"), conftest)
source = (ROOT / "rules/benchmark/tests/test_rule_regressions.py").read_text()
source = source.replace("ROOT = Path(__file__).resolve().parents[3]", "")
tests = {"ROOT": ROOT}
exec(compile(source, "test_rule_regressions.py", "exec"), tests)

# --- the recorder ----------------------------------------------------------------------------
files = {}  # repository-relative path -> bytes, for files read by the current test
real_read_bytes = Path.read_bytes


def read_bytes(self):
    data = real_read_bytes(self)
    try:
        files[str(self.resolve().relative_to((ROOT / "tests_data").resolve()))] = data
    except ValueError:
        pass
    return data


Path.read_bytes = read_bytes
blobs = []  # distinct byte prefixes, each stored once as run-length segments; whole zip inputs
blob_index = {}
groups = {}  # test name -> list of cases
current = {}


def segments(prefix):
    out, i = [], 0
    while i < len(prefix):
        j = i
        while j < len(prefix) and prefix[j] == prefix[i]:
            j += 1
        if j - i >= 16:
            out.append([f"{prefix[i]:02x}", j - i])
            i = j
        else:
            if out and isinstance(out[-1], str):
                out[-1] += f"{prefix[i]:02x}"
            else:
                out.append(f"{prefix[i]:02x}")
            i += 1
    return out


def patches(base, content):
    diff = [i for i in range(len(content)) if base[i] != content[i]]
    if len(diff) > 64:
        return None
    runs = []
    for i in diff:
        if runs and i == runs[-1][0] + len(runs[-1][1]) // 2:
            runs[-1][1] += f"{content[i]:02x}"
        else:
            runs.append([i, f"{content[i]:02x}"])
    return runs


def is_zip(data):
    return data.startswith(b"PK\x03\x04") or data.startswith(b"PK\x05\x06")


def blob(data):
    # Zip facts read the tail too, so a zip input is stored whole.
    key = bytes(data) if is_zip(data) else data[:PREFIX]
    if key not in blob_index:
        blob_index[key] = len(blobs)
        blobs.append(segments(key))
    return blob_index[key]


def encode(content):
    content = bytes(content)
    size = len(content)
    prefix = content[:PREFIX]
    candidates = [("file", path, data) for path, data in files.items()]
    candidates += [("blob", None, base) for base in current["seen"] if not is_zip(base)]
    for kind, path, base in candidates:
        head = base[:PREFIX]
        origin = {"file": path} if kind == "file" else {"blob": blob(base)}
        if len(prefix) <= len(head) and prefix == head[: len(prefix)] and (kind == "blob" or size <= len(base)):
            return origin | {"len": size}
        if len(prefix) == len(head):
            runs = patches(head, prefix)
            if runs is not None:
                return origin | {"len": size, "patch": runs}
    current["seen"].append(content)
    return {"blob": blob(content), "len": size}


class Recorded:
    def __init__(self, content):
        self.input = encode(content)

    def __eq__(self, expected):
        groups.setdefault(current["name"], []).append(self.input | {"labels": sorted(expected)})
        return True

    def __contains__(self, label):
        groups.setdefault(current["name"], []).append(self.input | {"absent": label})
        return False


scan_rules = Recorded
fixtures = {
    "scan_rules": scan_rules,
    "reviewed_binary_headers": conftest["reviewed_binary_headers"](),
    "reviewed_binary_header_variants": conftest["reviewed_binary_header_variants"](),
}
fixtures["dex_only_archives"] = conftest["dex_only_archives"](fixtures["reviewed_binary_headers"])

skipped = []
for name, fn in list(tests.items()):
    if not (name.startswith("test_") and callable(fn)):
        continue
    if name in NATIVE_ENGINE or name in NATIVE_RUST:
        skipped.append(name)
        continue
    grids = fn.__dict__.get("_params", [])
    axes = []
    for names, values in grids:
        keys = [n.strip() for n in names.split(",")]
        axes.append([dict(zip(keys, v if len(keys) > 1 else (v,))) for v in values])
    for combo in itertools.product(*axes) if axes else [()]:
        kwargs = {k: v for part in combo for k, v in part.items()}
        if name == "test_prefix_signatures_for_labels_the_model_lacks_or_misses" and kwargs["label"] == "asf":
            continue  # taxonomy_asf is parked: the evaluation dataset refutes it
        args = fn.__code__.co_varnames[: fn.__code__.co_argcount]
        for arg in args:
            if arg in fixtures:
                kwargs[arg] = fixtures[arg]
            elif arg == "tmp_path":
                raise SystemExit(f"{name} needs tmp_path")
        current["name"] = name
        current["seen"] = []
        files.clear()
        fn(**kwargs)

def merged(cases):
    """Consecutive truncations of one source with one expectation become a length range."""
    out = []
    for case in cases:
        last = out[-1] if out else None
        key = {k: v for k, v in case.items() if k != "len"}
        if (last and "patch" not in case and "lens" in last
                and {k: v for k, v in last.items() if k != "lens"} == key
                and case["len"] == last["lens"][1] + 1):
            last["lens"][1] = case["len"]
        elif "patch" not in case:
            out.append(key | {"lens": [case["len"], case["len"]]})
        else:
            out.append(case)
    return out


groups = {name: merged(cases) for name, cases in groups.items()}
def dump(value):
    return json.dumps(value, separators=(",", ":"))


lines = ["{", f' "source": {dump(SOURCE)},', ' "blobs": [']
lines += [f"  {dump(b)}," for b in blobs]
lines[-1] = lines[-1].rstrip(",")
lines += [" ],", ' "tests": {']
for index, (name, cases) in enumerate(groups.items()):
    lines.append(f"  {dump(name)}: [")
    lines += [f"   {dump(c)}," for c in cases]
    lines[-1] = lines[-1].rstrip(",")
    lines.append("  ]," if index + 1 < len(groups) else "  ]")
lines += [" }", "}"]
OUT.write_text("\n".join(lines) + "\n")
print(sum(c["lens"][1] - c["lens"][0] + 1 if "lens" in c else 1 for g in groups.values() for c in g), "cases in",
      sum(map(len, groups.values())), "entries,", len(blobs), "blobs;", OUT.stat().st_size, "bytes; skipped:", ", ".join(sorted(skipped)))
