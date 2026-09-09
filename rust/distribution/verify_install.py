# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Install locally built cargo-dist artifacts and exercise the installed CPU backend."""

import functools
import http.server
import json
import os
import platform
import subprocess
import sys
import tempfile
import threading
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def verify(artifacts):
    handler = functools.partial(
        http.server.SimpleHTTPRequestHandler, directory=str(artifacts)
    )
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    worker = threading.Thread(target=server.serve_forever, daemon=True)
    worker.start()
    try:
        with tempfile.TemporaryDirectory(prefix="magika-installed-") as temporary:
            env = dict(os.environ)
            env.pop("MAGIKA_RUNTIME_DIR", None)
            env["MAGIKA_CLI_DOWNLOAD_URL"] = f"http://127.0.0.1:{server.server_port}"
            env["MAGIKA_CLI_UNMANAGED_INSTALL"] = temporary
            windows = platform.system() == "Windows"
            installer = artifacts / (
                "magika-cli-installer.ps1" if windows else "magika-cli-installer.sh"
            )
            command = (
                ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]
                if windows
                else ["sh"]
            )
            subprocess.run([*command, str(installer)], env=env, check=True)
            executable = Path(temporary) / ("magika.exe" if windows else "magika")
            # Explicit CPU ensures an absent plugin cannot be hidden by a GPU path.
            result = subprocess.check_output(
                [
                    str(executable),
                    "--backend=cpu",
                    "--jsonl",
                    str(ROOT / "tests_data/basic/rust/code.rs"),
                ],
                env=env,
                text=True,
            )
            assert json.loads(result)["result"]["value"]["output"]["label"] == "rust", (
                result
            )
            print(
                "Clean installation: CPU classification passed without runtime overrides"
            )
    finally:
        server.shutdown()
        server.server_close()
        worker.join()


if __name__ == "__main__":
    verify(Path(sys.argv[1]).resolve())
