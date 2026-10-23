# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import os
import sys

import pytest

from magika_datasets.benchmark.process import run_cli, sample_order


def test_seeded_order_uses_whole_cycles():
    values = list(range(10))
    assert sample_order(values, 20, 7)[:5] == sample_order(values, 5, 7)
    assert set(sample_order(values, 20, 7)[:10]) == set(values)
    assert sample_order(values, 20, 7) != sample_order(values, 20, 8)


def test_subprocess_failures_and_timeouts_raise():
    with pytest.raises(RuntimeError, match="exited"):
        run_cli([sys.executable, "-c", "raise SystemExit(3)"], os.environ.copy(), 5)
    with pytest.raises(RuntimeError, match="exceeded"):
        run_cli([sys.executable, "-c", "import time; time.sleep(60)"], os.environ.copy(), 0.05)
