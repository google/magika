# Exploratory launchers

These standalone C/Rust launchers produced longer process timings than the actual
Hyperfine executor and are not used for the final accounting table. The pilot JSON
came from the initial C launcher with child-side file opens, before the retained C
source moved file creation outside the timing window. It is exploratory evidence,
not an exact-source reproduction receipt. The final method instruments Hyperfine's
existing timer directly; see the parent README and results.
