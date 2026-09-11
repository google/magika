# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Independent, append-only detector observations. Never changes sample labels."""

import base64
import concurrent.futures
import fcntl
import hashlib
import json
import os
import signal
import subprocess
import tempfile
import time
from collections import Counter
from pathlib import Path

TOOLS = frozenset(
    {"file", "trid", "magika", "exiftool", "puremagic", "file-type", "siegfried", "fido"}
)


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def bounded_command(argv, timeout=20, max_output=262144, cwd=None):
    """Capture original bytes; kill the entire subprocess group on timeout/overflow."""
    started = time.monotonic()
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        try:
            process = subprocess.Popen(
                argv,
                stdout=out,
                stderr=err,
                cwd=cwd,
                stdin=subprocess.DEVNULL,
                start_new_session=True,
            )
        except OSError as exc:
            return {"status": "unavailable", "error": str(exc), "elapsed_ms": 0}
        status = "ok"
        while process.poll() is None:
            if time.monotonic() - started > timeout:
                status = "timeout"
            elif os.fstat(out.fileno()).st_size + os.fstat(err.fileno()).st_size > max_output:
                status = "output_limit"
            if status != "ok":
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                break
            time.sleep(0.01)
        code = process.wait()
        size = os.fstat(out.fileno()).st_size + os.fstat(err.fileno()).st_size
        if status == "ok" and size > max_output:
            status = "output_limit"
        if status == "ok" and code != 0:
            status = "error"
        out.seek(0)
        err.seek(0)
        stdout, stderr = out.read(max_output), err.read(max_output)
        return {
            "status": status,
            "exit_code": code,
            "elapsed_ms": round((time.monotonic() - started) * 1000, 3),
            "stdout": stdout.decode("utf-8", errors="replace"),
            "stderr": stderr.decode("utf-8", errors="replace"),
            "stdout_base64": base64.b64encode(stdout).decode(),
            "stderr_base64": base64.b64encode(stderr).decode(),
            "output_truncated": size > max_output,
        }


def tool_identity(tool):
    """Hash runtime/signature artifacts, supplied metadata, and runner implementation."""
    identity = {**tool, "runner_sha256": digest(__file__), "artifact_sha256": {}}
    for path in tool.get("artifacts", []):
        p = Path(path)
        if p.is_dir():
            files = sorted(
                x
                for x in p.rglob("*")
                if x.is_file() and "__pycache__" not in x.parts and ".git" not in x.parts
            )
            h = hashlib.sha256()
            for f in files:
                h.update(str(f.relative_to(p)).encode() + b"\0" + digest(f).encode())
            identity["artifact_sha256"][path] = h.hexdigest()
        elif p.is_file():
            identity["artifact_sha256"][path] = digest(p)
        else:
            identity["artifact_sha256"][path] = None
    key = hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest()
    return key, identity


def observe(record, tool, key):
    base = {
        "sha256": record["sha256"],
        "tool": tool["id"],
        "tool_identity": key,
        "basis": "local_bytes",
        "observed_at": time.time(),
    }
    path = Path(record["path"]).resolve()
    if not path.is_file() or digest(path) != record["sha256"]:
        return {**base, "status": "input_error", "error": "Missing or mismatched SHA256"}
    if tool.get("unavailable_reason"):
        return {**base, "status": "unavailable", "error": tool["unavailable_reason"]}
    argv = [a.replace("{path}", str(path)) for a in tool["argv"]]
    return {
        **base,
        **bounded_command(
            argv,
            tool.get("timeout_seconds", 20),
            tool.get("max_output_bytes", 262144),
            tool.get("cwd"),
        ),
    }


def collect(records, tools, output, *, workers=8, progress=lambda report: None):
    if workers < 1 or len({t["id"] for t in tools}) != len(tools):
        raise ValueError("positive workers and unique tool IDs required")
    output = Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    identities = {t["id"]: tool_identity(t) for t in tools}
    counts, resumed = Counter(), 0
    with output.with_suffix(".lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        done = set()
        if output.exists():
            # A killed append may leave only the last JSON record incomplete.
            with output.open("rb+") as stream:
                good_end = 0
                while line := stream.readline():
                    if not line.endswith(b"\n"):
                        stream.truncate(good_end)
                        break
                    r = json.loads(line)
                    done.add((r["sha256"], r["tool_identity"]))
                    good_end = stream.tell()
        output.with_suffix(".tools.json").write_text(
            json.dumps({key: value for key, value in identities.values()}, indent=2) + "\n"
        )
        with (
            output.open("a", buffering=1) as stream,
            concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool,
        ):
            for tool in tools:
                key = identities[tool["id"]][0]
                pending = [r for r in records if (r["sha256"], key) not in done]
                resumed += len(records) - len(pending)
                for result in pool.map(lambda r: observe(r, tool, key), pending):
                    stream.write(json.dumps(result, sort_keys=True) + "\n")
                    counts[tool["id"] + ":" + result["status"]] += 1
                    if sum(counts.values()) % 100 == 0:
                        progress(
                            {
                                "new": sum(counts.values()),
                                "resumed": resumed,
                                "counts": dict(counts),
                            }
                        )
    return {
        "new": sum(counts.values()),
        "resumed": resumed,
        "counts": dict(counts),
        "files": len(records),
        "tools": len(tools),
    }
