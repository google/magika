## Description

This PR adds a Windows GUI application for Magika, providing an intuitive graphical interface for file type detection.

## Type of Change

- [x] New feature (non-breaking change which adds functionality)
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [x] Documentation update

## Changes Made

### New Features
- Semi-transparent floating window interface for Windows
- Drag-and-drop file analysis support
- Multi-file batch processing
- Detailed results display with confidence scores
- Real-time progress tracking
- Graceful error handling per file

### Files Added
- `python/src/magika/gui/magika_gui.py` - Main GUI implementation
- `python/src/magika/gui/__init__.py` - Module initialization
- `python/magika-gui.py` - Launcher script
- `docs/GUI_WINDOWS.md` - User documentation
- `docs/GUI_DEVELOPMENT.md` - Developer guide

### Files Modified
- `README.md` - Added GUI section
- `python/pyproject.toml` - Added optional dependencies and entry point

## Testing

- [x] Tested on Windows 10
- [x] Tested with single file
- [x] Tested with multiple files
- [x] Tested error handling
- [x] Tested with various file types
- [x] Tested drag-and-drop functionality
- [x] Tested file browser
- [x] Window dragging works correctly
- [x] Always-on-top behavior works
- [x] Progress updates display correctly

## Documentation

- [x] Code comments added
- [x] Docstrings added for public methods
- [x] User documentation created (GUI_WINDOWS.md)
- [x] Developer guide created (GUI_DEVELOPMENT.md)
- [x] README updated

## Dependencies

- Required: None (uses Python standard library Tkinter)
- Optional: `tkinterdnd2>=0.3.0` (for enhanced drag-and-drop)

## Breaking Changes

None - This is a purely additive feature.

## Screenshots

(Will be added after initial review - placeholder created in assets/)

## Checklist

- [x] My code follows the style guidelines of this project
- [x] I have performed a self-review of my own code
- [x] I have commented my code, particularly in hard-to-understand areas
- [x] I have made corresponding changes to the documentation
- [x] My changes generate no new warnings
- [x] I have added tests that prove my fix is effective or that my feature works
- [x] New and existing unit tests pass locally with my changes
- [x] Any dependent changes have been merged and published
- [ ] I have signed the Google CLA

## Additional Notes

- The GUI is designed to be lightweight and minimalist
- Uses threading to prevent UI blocking during analysis
- Compatible with Python 3.8+ (matching Magika requirements)
- Can be packaged as standalone executable with PyInstaller
- No changes to existing CLI or Python API

## Questions for Reviewers

1. Should this be included in the main package or distributed separately?
2. Are there any Google-specific branding guidelines to follow?
3. Would you like any additional features before merging?

## Related Issues

Addresses the need for a graphical interface on Windows platforms.
