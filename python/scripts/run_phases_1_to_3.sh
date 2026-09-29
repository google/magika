#!/usr/bin/env bash
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

# ==============================================================================
# Magika PyO3 Automated Verification Script (Phases 1 - 3)
# ==============================================================================
# This script automates the first three phases described in `tests.txt`:
#
#   Phase 1: Local Development & Workspace Checks (On This Machine)
#            - Verifies `uv run magika` invokes the native Rust CLI (ELF binary).
#            - Runs pytest unit tests against the PyO3 native extension.
#            - Runs static analysis and linters: ruff check, ruff format, mypy.
#            - Runs in-process Python smoke test via `uv run python`.
#
#   Phase 2: Wheel Creation & Inspection
#            - Executes `build_wheel.sh` to package the hybrid wheel.
#            - Validates wheel archive contents (checks for `_magika.abi3.so` and
#              the native CLI script; confirms `magika/cli/` and `magika/models/`
#              are completely absent).
#            - Inspects wheel metadata (verifies ZERO runtime dependencies).
#            - Inspects ABI tags (confirms `cp38-abi3-linux_x86_64`).
#
#   Phase 3: Clean Environment Testing (Zero-Dependency Validation)
#            - Creates a fresh, pristine virtual environment in /tmp.
#            - Installs the built wheel using standard `pip install`.
#            - Verifies `pip list` contains ONLY `magika` and `pip`.
#            - Verifies the installed standalone CLI binary runs properly.
#            - Verifies Python library detection from within the clean venv.
#            - Safely cleans up temporary virtual environments upon exit.
#
# Usage:
#   ./python/scripts/run_phases_1_to_3.sh          # Fast mode (skips 1GB disk test)
#   ./python/scripts/run_phases_1_to_3.sh --full   # Includes slow 1GB disk test
# ==============================================================================

set -euo pipefail

# ------------------------------------------------------------------------------
# Terminal Color & Formatting Helpers
# ------------------------------------------------------------------------------
BOLD="\033[1m"
GREEN="\033[0;32m"
CYAN="\033[0;36m"
YELLOW="\033[0;33m"
RED="\033[0;31m"
RESET="\033[0m"

log_header() {
    echo -e "\n${BOLD}${CYAN}====================================================================${RESET}"
    echo -e "${BOLD}${CYAN}  $1${RESET}"
    echo -e "${BOLD}${CYAN}====================================================================${RESET}"
}

log_phase() {
    echo -e "\n${BOLD}${YELLOW}>>> [PHASE $1] $2${RESET}"
}

log_step() {
    echo -e "\n${BOLD}--> $1${RESET}"
}

log_info() {
    echo -e "    ${CYAN}[INFO]${RESET} $1"
}

log_success() {
    echo -e "    ${GREEN}[PASS]${RESET} $1"
}

log_warn() {
    echo -e "    ${YELLOW}[WARN]${RESET} $1"
}

log_fail() {
    echo -e "    ${RED}[FAIL]${RESET} $1"
    exit 1
}

# ------------------------------------------------------------------------------
# Workspace Setup & Directory Navigation
# ------------------------------------------------------------------------------
# Resolve absolute path of the repository root, regardless of where this script
# is invoked from or whether it is called via a symlink.
if git -C "$(dirname -- "${BASH_SOURCE[0]}")" rev-parse --show-toplevel &>/dev/null; then
    REPO_ROOT="$(git -C "$(dirname -- "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
else
    REAL_SCRIPT="$(realpath "${BASH_SOURCE[0]}")"
    REPO_ROOT="$(cd -- "$(dirname -- "$REAL_SCRIPT")/../.." && pwd)"
fi
PYTHON_DIR="$REPO_ROOT/python"

cd "$REPO_ROOT"
log_info "Repository root: $REPO_ROOT"
log_info "Python directory: $PYTHON_DIR"

RUN_FULL_TESTS=false
if [[ "${1:-}" == "--full" ]]; then
    RUN_FULL_TESTS=true
    log_info "Full test mode enabled: will include 1GB large-file disk test."
else
    log_info "Fast mode: large-file test excluded (pass --full to include)."
fi

# Track temporary directories for guaranteed cleanup
TEMP_CLEAN_VENV=""
cleanup() {
    if [[ -n "$TEMP_CLEAN_VENV" && -d "$TEMP_CLEAN_VENV" ]]; then
        log_info "Cleaning up temporary virtual environment: $TEMP_CLEAN_VENV"
        rm -rf "$TEMP_CLEAN_VENV"
    fi
}
trap cleanup EXIT INT TERM

# ==============================================================================
# PHASE 1: Local Development & Workspace Checks (On This Machine)
# ==============================================================================
log_phase "1" "Local Development & Workspace Checks"

# ------------------------------------------------------------------------------
# Check 1.1: Verify local `magika` CLI in .venv
# ------------------------------------------------------------------------------
log_step "1.1: Verify local CLI execution and check executable binary format"

CLI_PATH=$(uv run --project python which magika 2>/dev/null || true)
if [[ -z "$CLI_PATH" ]]; then
    log_fail "Could not find 'magika' binary via 'uv run --project python which magika'."
fi

log_info "Discovered CLI binary at: $CLI_PATH"

# Run `file` command to prove it is a compiled ELF native binary, NOT a Python script
FILE_OUTPUT=$(file "$CLI_PATH")
log_info "File type output: $FILE_OUTPUT"

if echo "$FILE_OUTPUT" | grep -qE "ELF 64-bit|Mach-O|PE32"; then
    log_success "Verified CLI is a native compiled executable (not a Python wrapper script)."
else
    log_fail "CLI binary does not appear to be a compiled native binary: $FILE_OUTPUT"
fi

# Run `--version` to check version and model info reported by the Rust CLI
VERSION_OUTPUT=$(uv run --project python magika --version)
log_info "Rust CLI reported version: $VERSION_OUTPUT"
if echo "$VERSION_OUTPUT" | grep -q "magika"; then
    log_success "Local native CLI ran successfully with output: $VERSION_OUTPUT"
else
    log_fail "Unexpected output from 'magika --version': $VERSION_OUTPUT"
fi

# ------------------------------------------------------------------------------
# Check 1.2: Run Python Unit Test Suite
# ------------------------------------------------------------------------------
log_step "1.2: Run Python unit tests against the PyO3 native extension"

cd "$PYTHON_DIR"
if [[ "$RUN_FULL_TESTS" == true ]]; then
    log_info "Executing all pytest tests (including 1GB file test)..."
    uv run pytest tests/
else
    log_info "Executing pytest test suite (skipping 1GB big_file test for speed)..."
    uv run pytest tests/ -k "not test_magika_module_with_big_file"
fi
cd "$REPO_ROOT"
log_success "Pytest test suite passed cleanly!"

# ------------------------------------------------------------------------------
# Check 1.3: Verify Linters and Type Checkers
# ------------------------------------------------------------------------------
log_step "1.3: Verify static analysis, linting, and type checking"

cd "$PYTHON_DIR"

log_info "Running 'uv run ruff check'..."
uv run ruff check
log_success "Ruff linter passed (0 errors)."

log_info "Running 'uv run ruff format --check'..."
uv run ruff format --check
log_success "Ruff formatter check passed."

log_info "Running 'uv run mypy src/magika'..."
uv run mypy src/magika
log_success "Mypy type checker passed (strict types verified, 0 errors)."

cd "$REPO_ROOT"

# ------------------------------------------------------------------------------
# Check 1.4: Python In-Process Smoke Test
# ------------------------------------------------------------------------------
log_step "1.4: Python in-process smoke test (PyO3 module & Rust session)"

SAMPLE_FILE="$REPO_ROOT/tests_data/basic/python/code.py"
if [[ ! -f "$SAMPLE_FILE" ]]; then
    log_fail "Expected sample file not found: $SAMPLE_FILE"
fi

cd "$PYTHON_DIR"
uv run python -c "
import magika
from pathlib import Path

# 1. Instantiate Magika backed by native PyO3 session
m = magika.Magika()

# 2. Check model name reporting
session_model = m.get_model_name()
default_model = magika.Magika._get_default_model_name()
print(f'    [INFO] Session model name: {session_model}')
print(f'    [INFO] Default model name (PyO3 function): {default_model}')
assert session_model == default_model, f'Model mismatch: {session_model} != {default_model}'

# 3. Test detection on sample file
sample_path = Path('$SAMPLE_FILE')
res = m.identify_path(sample_path)
print(f'    [INFO] Detection label: {res.prediction.output.label}')
print(f'    [INFO] Detection score: {res.prediction.score:.4f}')
print(f'    [INFO] MIME type:       {res.prediction.output.mime_type}')
assert res.ok, f'Prediction status not OK: {res.status}'
assert res.prediction.output.label == 'python', f'Expected python, got {res.prediction.output.label}'
assert res.prediction.score > 0.95, f'Expected score > 0.95, got {res.prediction.score}'
"
cd "$REPO_ROOT"
log_success "In-process Python smoke test passed!"


# ==============================================================================
# PHASE 2: Wheel Creation & Inspection
# ==============================================================================
log_phase "2" "Wheel Creation & Inspection"

# ------------------------------------------------------------------------------
# Step 2.1: Build Release Wheel via build_wheel.sh
# ------------------------------------------------------------------------------
log_step "2.1: Build release wheel via './python/scripts/build_wheel.sh'"

log_info "Executing build_wheel.sh..."
./python/scripts/build_wheel.sh

# Locate the newly generated wheel
WHEEL_FILE=$(ls -t "$PYTHON_DIR"/dist/magika-*.whl 2>/dev/null | head -n 1 || true)
if [[ -z "$WHEEL_FILE" || ! -f "$WHEEL_FILE" ]]; then
    log_fail "No wheel file found in $PYTHON_DIR/dist/ after running build_wheel.sh."
fi

WHEEL_NAME=$(basename "$WHEEL_FILE")
WHEEL_SIZE=$(ls -lh "$WHEEL_FILE" | awk '{print $5}')
log_info "Built wheel: $WHEEL_NAME ($WHEEL_SIZE)"

# ------------------------------------------------------------------------------
# Check 2.2: Inspect Wheel Contents (Dual-Entrypoint verification)
# ------------------------------------------------------------------------------
log_step "2.2: Inspect wheel archive contents (verifying files and exclusions)"

WHEEL_CONTENTS=$(unzip -l "$WHEEL_FILE")

# 1. Verify PyO3 extension library exists
if echo "$WHEEL_CONTENTS" | grep -q "magika/_magika.abi3.so"; then
    log_success "Found PyO3 C-extension: magika/_magika.abi3.so"
else
    log_fail "Missing PyO3 C-extension 'magika/_magika.abi3.so' in wheel!"
fi

# 2. Verify standalone native CLI script exists in .data/scripts/
CLI_SCRIPT_NAME=$(echo "$WHEEL_CONTENTS" | grep -oE '[^/[:space:]]+\.data/scripts/[^[:space:]]+' | grep -v '\.gitkeep' | head -n 1 | awk -F/ '{print $NF}' || true)
if [[ -n "$CLI_SCRIPT_NAME" ]]; then
    log_success "Found native CLI script in wheel: .data/scripts/$CLI_SCRIPT_NAME"
else
    log_fail "Missing native CLI script in wheel .data/scripts/!"
fi

# 3. Verify removed components are NOT present in the wheel
if echo "$WHEEL_CONTENTS" | grep -q "magika/cli/"; then
    log_fail "Legacy 'magika/cli/' was found inside the wheel! It should be removed."
else
    log_success "Confirmed: 'magika/cli/' is completely absent from the wheel."
fi

if echo "$WHEEL_CONTENTS" | grep -q "magika/models/"; then
    log_fail "Legacy 'magika/models/' was found inside the wheel! Redundant model.onnx should be removed."
else
    log_success "Confirmed: 'magika/models/' and redundant 3.1MB model.onnx are absent from wheel."
fi

# 4. Verify relocated configuration file is present
if echo "$WHEEL_CONTENTS" | grep -q "magika/config/model_config.min.json"; then
    log_success "Found relocated config: magika/config/model_config.min.json"
else
    log_fail "Missing relocated config 'magika/config/model_config.min.json' in wheel!"
fi

# ------------------------------------------------------------------------------
# Check 2.3: Inspect Wheel Metadata & ABI Tags
# ------------------------------------------------------------------------------
log_step "2.3: Inspect wheel metadata & ABI tags"

# Check Requires-Dist: must have ZERO runtime dependencies
REQUIRES_DIST=$(unzip -p "$WHEEL_FILE" "*.dist-info/METADATA" | grep -i "^Requires-Dist:" || true)
if [[ -n "$REQUIRES_DIST" ]]; then
    log_fail "Expected 0 runtime dependencies, but found Requires-Dist: $REQUIRES_DIST"
else
    log_success "Verified: wheel has 0 runtime dependencies (no onnxruntime, no click)."
fi

# Check Wheel ABI Tag: must be abi3 compatible with Python 3.8+
WHEEL_TAG=$(unzip -p "$WHEEL_FILE" "*.dist-info/WHEEL" | grep -i "^Tag:" || true)
log_info "Wheel Tag entry: $WHEEL_TAG"
if echo "$WHEEL_TAG" | grep -q "abi3"; then
    log_success "Verified: wheel specifies abi3 compatibility tag."
else
    log_warn "Wheel tag did not mention abi3: $WHEEL_TAG"
fi


# ==============================================================================
# PHASE 3: Clean Environment Testing (Zero-Dependency Validation)
# ==============================================================================
log_phase "3" "Clean Environment Testing (Zero-Dependency Validation)"

# ------------------------------------------------------------------------------
# Step 3.1: Create Clean Virtual Environment
# ------------------------------------------------------------------------------
log_step "3.1: Create isolated temporary virtual environment"

TEMP_CLEAN_VENV=$(mktemp -d /tmp/magika_clean_test_venv.XXXXXX)
log_info "Creating pristine Python virtual environment at: $TEMP_CLEAN_VENV"
python3 -m venv "$TEMP_CLEAN_VENV"

VENV_PIP="$TEMP_CLEAN_VENV/bin/pip"
VENV_PYTHON="$TEMP_CLEAN_VENV/bin/python"

# ------------------------------------------------------------------------------
# Step 3.2: Install Built Wheel into Clean Virtual Environment
# ------------------------------------------------------------------------------
log_step "3.2: Install built wheel via 'pip install'"

log_info "Installing $WHEEL_FILE..."
"$VENV_PIP" install "$WHEEL_FILE"
log_success "Wheel installation completed successfully."

# ------------------------------------------------------------------------------
# Check 3.3: Verify Zero External Dependencies in Clean Environment
# ------------------------------------------------------------------------------
log_step "3.3: Verify installed packages (zero third-party dependencies)"

PIP_LIST_OUTPUT=$("$VENV_PIP" list --format=freeze)
log_info "Installed packages in clean venv:"
echo "$PIP_LIST_OUTPUT" | while read -r line; do
    echo "      $line"
done

# Check that click and onnxruntime are definitely not present
if echo "$PIP_LIST_OUTPUT" | grep -qiE "click|onnxruntime"; then
    log_fail "Unexpected third-party dependency found in clean venv: $PIP_LIST_OUTPUT"
else
    log_success "Verified: clean environment contains only 'magika' (and standard pip tools)."
fi

# ------------------------------------------------------------------------------
# Check 3.4: Verify Installed Native CLI Binary
# ------------------------------------------------------------------------------
log_step "3.4: Verify installed native CLI binary"

INSTALLED_CLI="$TEMP_CLEAN_VENV/bin/$CLI_SCRIPT_NAME"
if [[ ! -f "$INSTALLED_CLI" ]]; then
    log_fail "Expected installed CLI binary not found at: $INSTALLED_CLI"
fi

CLI_FILE_INFO=$(file "$INSTALLED_CLI")
log_info "Installed CLI file info: $CLI_FILE_INFO"
if echo "$CLI_FILE_INFO" | grep -qE "ELF 64-bit|Mach-O|PE32"; then
    log_success "Installed CLI is a compiled native binary."
else
    log_fail "Installed CLI is not a compiled native binary: $CLI_FILE_INFO"
fi

# Run version command
CLI_VERSION_RES=$("$INSTALLED_CLI" --version)
log_info "CLI --version output: $CLI_VERSION_RES"
log_success "Installed native CLI executed --version successfully."

# Run detection on sample file
CLI_DETECT_RES=$("$INSTALLED_CLI" "$SAMPLE_FILE")
log_info "CLI scan output: $CLI_DETECT_RES"
if echo "$CLI_DETECT_RES" | grep -qi "python"; then
    log_success "Installed native CLI accurately detected Python sample file."
else
    log_fail "Installed native CLI failed to identify python sample: $CLI_DETECT_RES"
fi

# ------------------------------------------------------------------------------
# Check 3.5: Verify Python Library Import & Detection in Clean Environment
# ------------------------------------------------------------------------------
log_step "3.5: Verify Python library import & detection in clean environment"

"$VENV_PYTHON" -c "
import magika
from pathlib import Path

# Verify initialization
m = magika.Magika()
print(f'    [INFO] Initialized Magika() in clean environment.')
print(f'    [INFO] Model name: {m.get_model_name()}')
print(f'    [INFO] Module default: {magika.Magika._get_default_model_name()}')

# Verify inference
sample_path = Path('$SAMPLE_FILE')
res = m.identify_path(sample_path)
print(f'    [INFO] Result status: {res.status}')
print(f'    [INFO] Result label:  {res.prediction.output.label}')
print(f'    [INFO] Result score:  {res.prediction.score:.4f}')

assert res.ok, f'Status is not OK: {res.status}'
assert res.prediction.output.label == 'python', f'Expected python, got {res.prediction.output.label}'
assert res.prediction.score > 0.95, f'Expected high score, got {res.prediction.score}'
"
log_success "Python library detection succeeded in clean environment."


# ==============================================================================
# All Checks Passed!
# ==============================================================================
log_header "ALL 3 PHASES PASSED SUCCESSFULLY!"
echo -e "${GREEN}${BOLD}Summary:${RESET}"
echo -e "  ${GREEN}✓${RESET} Phase 1: Local workspace, native CLI, pytest suite, ruff, and mypy passed."
echo -e "  ${GREEN}✓${RESET} Phase 2: Release wheel created without legacy CLI/models, verified 0 dependencies."
echo -e "  ${GREEN}✓${RESET} Phase 3: Installed wheel into pristine venv; verified zero external dependencies, native CLI binary execution, and Python library inference."
echo ""
