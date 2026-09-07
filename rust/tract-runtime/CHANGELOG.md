# Changelog

## 0.0.1-dev

### Minor

- Add the shared tract inference runtime with CPU, Metal, and optional CUDA backends.
- Load the checked model artifact and prepare fixed batch plans shared across sessions.
- Validate every resident GPU batch plan against a stored CPU reference at startup.
