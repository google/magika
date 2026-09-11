# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Bounded Git subprocess access for pinned repository objects."""

import subprocess


def git(repo, *args):
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True)
    if result.returncode:
        raise ValueError(
            f"git metadata unavailable for {repo}: {result.stderr.decode(errors='replace')[:300]}"
        )
    return result.stdout
