# Sync the content types knowledge base and regenerate derived language bindings.
sync-kb:
    uv run scripts/sync_kb.py
    uv run python/scripts/sync.py python
    uv run python/scripts/sync.py js
    cd rust && ./sync.sh
