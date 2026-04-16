# Add Windows GUI Application for Magika

## Overview

This PR introduces a lightweight, user-friendly Windows GUI for Magika that provides an intuitive graphical interface for file type detection. The GUI complements the existing CLI and Python API, making Magika accessible to users who prefer graphical interfaces.

## Motivation

While Magika's CLI is powerful, many Windows users prefer graphical interfaces for file analysis tasks. This GUI provides:
- Quick drag-and-drop file analysis
- Visual display of detection results
- Easy access without terminal commands
- Always-accessible floating window interface

## Features

### Core Functionality
- **Drag & Drop Support**: Drop files directly onto the window for instant analysis
- **File Browser**: Click-to-browse interface for file selection
- **Multi-file Analysis**: Process multiple files in a single operation
- **Detailed Results**: Shows file type, MIME type, label, and confidence scores

### UI/UX Design
- **Semi-transparent Floating Window**: Modern, minimalist design that stays on top
- **Draggable Interface**: Click and drag to reposition the window
- **Non-blocking Processing**: Background threading for responsive UI
- **Status Indicators**: Real-time feedback on processing status

### Technical Implementation
- Built with Python's Tkinter (included in standard Python distribution)
- Optional `tkinterdnd2` integration for enhanced drag-and-drop
- Follows existing Magika API and architecture
- Minimal dependencies for easy installation

## Installation

Users can install and run the GUI in multiple ways:

```bash
# Basic installation (with optional GUI support)
pip install magika tkinterdnd2

# Launch the GUI
python -m magika.gui
```

Or with optional dependencies:
```bash
pip install magika[gui]
```

## Files Added/Modified

### New Files
- `python/src/magika/gui/magika_gui.py` - Main GUI application
- `python/src/magika/gui/__init__.py` - GUI module initialization
- `python/magika-gui.py` - Launcher script
- `docs/GUI_WINDOWS.md` - Comprehensive documentation
- `assets/magika-gui-screenshot.txt` - Screenshot placeholder

### Modified Files
- `README.md` - Added GUI installation and quick start info
- `python/pyproject.toml` - Added optional GUI dependencies and entry point

## Documentation

Complete documentation is provided in `docs/GUI_WINDOWS.md`, including:
- Installation instructions
- Usage guide with examples
- Configuration options
- Troubleshooting section
- PyInstaller instructions for creating standalone executables

## Testing

The GUI has been tested with:
- Multiple file types (binary and text)
- Various file sizes
- Multi-file selection
- Window dragging and repositioning
- Background processing and status updates

## Compatibility

- **OS**: Windows (primary target), also compatible with macOS/Linux
- **Python**: 3.8+ (matches existing Magika requirements)
- **Dependencies**: 
  - Required: magika, tkinter (standard library)
  - Optional: tkinterdnd2 (for enhanced drag-and-drop)

## Design Decisions

1. **Tkinter Choice**: Selected for its inclusion in Python's standard library, minimizing additional dependencies
2. **Semi-transparent Window**: Provides modern aesthetic while maintaining visibility of desktop content
3. **Always-on-top**: Ensures quick access without searching through windows
4. **Background Threading**: Prevents UI freezing during file analysis
5. **Optional Dependencies**: tkinterdnd2 is optional; GUI works with basic file browser if not installed

## Future Enhancements

Potential improvements for future PRs:
- Batch processing with progress bars
- History/recent files functionality
- Export results to CSV/JSON
- Custom themes and color schemes
- Context menu integration (right-click file → Analyze with Magika)

## Screenshots

(TODO: Add actual screenshot once PR is reviewed - placeholder added in `assets/magika-gui-screenshot.txt`)

The GUI features:
- Clean header with minimize/close buttons
- Large drop zone for intuitive file input
- Scrollable results panel
- Status bar with real-time updates

## Checklist

- [x] Code follows project style and conventions
- [x] Apache 2.0 license headers included
- [x] Documentation provided
- [x] Compatible with existing Magika API
- [x] No breaking changes to existing functionality
- [x] Optional dependencies clearly marked
- [ ] CLA signed (will sign when prompted)

## Questions for Reviewers

1. Should the GUI be published as a separate package or included in the main `magika` package?
2. Are there any Google-specific UI guidelines to follow?
3. Should we add GUI telemetry/analytics (similar to CLI usage tracking)?
4. Would you like additional features before merge?

## Related Issues

This PR addresses the need for a graphical interface on Windows, making Magika more accessible to non-technical users and providing a quick-access tool for file analysis.

---

Thank you for considering this contribution! I'm happy to make any adjustments based on your feedback.
