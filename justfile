set positional-arguments

# Python package development tasks (`just python <recipe>` or `cd python && just <recipe>`).
mod python

[private]
default:
    @just --justfile "{{source_file()}}" --list --list-submodules

# Sync the content types knowledge base and regenerate derived language bindings.
sync-kb:
    uv run scripts/sync_kb.py
    uv run python/scripts/sync.py python
    uv run python/scripts/sync.py js
    cd rust && ./sync.sh

# Check that source files carry a valid copyright header.
check-copyright:
    uv run scripts/check_copyright.py

# Check documentation links and version references across Markdown files.
check-docs *args:
    uv run scripts/check_documentation.py "$@"
