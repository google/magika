set positional-arguments

# Python package development tasks (`just python <recipe>` or `cd python && just <recipe>`).
mod python

# Rust workspace tasks (`just rust <recipe>` or `cd rust && just <recipe>`).
mod rust

[private]
default:
    @just --justfile "{{source_file()}}" --list --list-submodules

# Sync the content types knowledge base, derived language bindings, and active model README.
sync-kb *args:
    uv run scripts/sync_kb.py "$@"

# Check that source files carry a valid copyright header.
check-copyright:
    uv run scripts/check_copyright.py

# Check documentation links and version references across Markdown files.
check-docs *args:
    uv run scripts/check_documentation.py "$@"
