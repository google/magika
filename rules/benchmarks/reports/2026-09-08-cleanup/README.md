# Startup cleanup verification

The committed spike is consolidated into the PR checkout. Mapped rules now use
the normal rules feature on Unix; `MAGIKA_RULES_MMAP=0` retains serialized diagnostics.
The distribution builder packages deferred CPU and optional GPU libraries; source
staging includes their loader and ABI. The bundle smoke test verifies CPU loading
and rules compilation from the assembled directory.

The loader diagnostic confirms rules-only loads neither backend and explicit CPU
loads no Metal backend/framework. Auto fallback and explicit GPU errors retain
their behavior. All serialized/mapped cache and packaging checks listed here pass.
The old spike worktree was clean and removed; its committed work and benchmark
measurements survive in this checkout. 363 historical experiment files are retained
byte-for-byte in `../../spikes/startup-spikes-2026-09-08.tar.gz` with a receipt.
The user-owned handoff remains unchanged and has a tracked archival copy.

The maintained build command is `python3 rust/build-runtime.py --gpu metal --output NEW_DIR`.
Standard installer/wheel release integration remains a release hold; these helper
bundles are the verified distribution for PR testing.
