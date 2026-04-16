# Magika GUI Development Guide

## Architecture

The Magika Windows GUI is built using Python's Tkinter library and follows a simple, maintainable architecture:

```
magika/
├── gui/
│   ├── __init__.py          # Module initialization, exports main()
│   └── magika_gui.py        # Main GUI implementation
└── magika-gui.py            # Launcher script (optional)
```

## Code Structure

### MagikaGUI Class

The main `MagikaGUI` class manages the entire application:

```python
class MagikaGUI:
    def __init__(self, root: tk.Tk)
    def setup_window(self)           # Window configuration
    def create_widgets(self)         # UI creation
    def setup_drag_and_drop(self)    # Drag-and-drop support
    def analyze_files(self, paths)   # File analysis logic
    def display_results(self, results) # Results rendering
```

### Key Components

1. **Window Management**
   - Semi-transparent overlay (`attributes('-alpha', 0.95)`)
   - Always-on-top behavior (`attributes('-topmost', True)`)
   - Frameless window (`overrideredirect(True)`)
   - Custom dragging implementation

2. **UI Elements**
   - Header frame with title and controls
   - Drop zone for file input
   - Results text widget with scrollbar
   - Status bar for feedback

3. **File Processing**
   - Background threading to prevent UI blocking
   - Thread-safe updates using `root.after()`
   - Error handling and user feedback

## Development Setup

### Prerequisites

```bash
# Install development dependencies
pip install magika tkinterdnd2

# For development mode
cd python
pip install -e .
```

### Running in Development

```bash
# From project root
python -m magika.gui

# Or directly
python python/magika-gui.py

# Or with full path
python python/src/magika/gui/magika_gui.py
```

## Testing

### Manual Testing Checklist

- [ ] Window launches successfully
- [ ] Window is semi-transparent
- [ ] Window can be dragged by title bar
- [ ] Minimize button works
- [ ] Close button works
- [ ] Drop zone is visible and clickable
- [ ] File browser opens on click
- [ ] Single file analysis works
- [ ] Multiple files can be analyzed
- [ ] Results display correctly
- [ ] Status updates appear
- [ ] Confidence scores are formatted properly
- [ ] Window stays on top of other applications
- [ ] No UI freezing during analysis
- [ ] Error messages display for invalid files

### Test Files

Use the `tests_data/` directory from the Magika repository:

```bash
# Test with various file types
python -m magika.gui
# Then select files from tests_data/basic/
```

### Error Scenarios

Test these error conditions:
- Launch GUI before Magika model is loaded
- Analyze non-existent file
- Analyze empty file
- Analyze very large file
- Analyze many files simultaneously

## Code Style

Follow the existing Magika Python code style:

```python
# Type hints
def analyze_files(self, file_paths: List[str]) -> None:
    pass

# Docstrings (Google style)
def display_results(self, results: List[Dict]) -> None:
    """Display analysis results in the text widget.

    Args:
        results: List of result dictionaries.
    """
    pass

# Formatting with ruff
ruff format python/src/magika/gui/
```

## Customization Guide

### Changing Window Transparency

```python
# In setup_window()
self.root.attributes('-alpha', 0.95)  # 0.0 to 1.0
```

### Adjusting Window Size

```python
# In setup_window()
self.root.geometry("400x500")  # width x height
```

### Modifying Colors

Current color scheme:
```python
BACKGROUND = '#1e1e1e'       # Main background
HEADER_BG = '#2d2d2d'        # Header background
DROP_ZONE_BG = '#2d2d2d'     # Drop zone background
RESULTS_BG = '#252525'       # Results panel background
TEXT_COLOR = '#e0e0e0'       # Primary text
ACCENT_COLOR = '#64B5F6'     # Highlighted text
SUCCESS_COLOR = '#4CAF50'    # Success status
ERROR_COLOR = '#f44336'      # Error status
WARNING_COLOR = '#FFC107'    # Warning status
```

### Adding New Features

Example: Add a "Clear Results" button

```python
def create_widgets(self):
    # ... existing code ...
    
    # Add clear button
    clear_btn = tk.Button(
        content_frame,
        text="Clear Results",
        command=self.clear_results,
        bg='#2d2d2d',
        fg='#ffffff',
        relief=tk.FLAT
    )
    clear_btn.pack(pady=5)

def clear_results(self):
    """Clear the results text widget."""
    self.results_text.config(state=tk.NORMAL)
    self.results_text.delete(1.0, tk.END)
    self.results_text.config(state=tk.DISABLED)
```

## Building Executables

### PyInstaller

Create a standalone Windows executable:

```bash
# Install PyInstaller
pip install pyinstaller

# Build executable
pyinstaller --onefile \
            --windowed \
            --name "Magika" \
            --icon=assets/icon.ico \
            --add-data "python/src/magika/models;magika/models" \
            python/magika-gui.py

# Output: dist/Magika.exe
```

### Distribution

For distribution, consider:
1. **Installer**: Use Inno Setup or NSIS for Windows installers
2. **Portable**: Provide a zip file with the executable
3. **Microsoft Store**: Package as MSIX for Store distribution

## Performance Optimization

### Current Optimizations

1. **Lazy Model Loading**: Model loads in background thread
2. **Non-blocking UI**: File analysis runs in separate thread
3. **Efficient Updates**: Use `root.after()` for thread-safe UI updates
4. **Limited Text Widget**: Disabled state when not editing to improve performance

### Potential Improvements

- Cache Magika instance across multiple analyses
- Implement result pagination for large file batches
- Add progress indicator for large files
- Use virtual scrolling for many results

## Troubleshooting

### Common Issues

**Issue**: Window doesn't appear
- Check if Python has GUI support: `python -m tkinter`
- Verify Tkinter is installed (usually included with Python)

**Issue**: Drag-and-drop not working
- Install tkinterdnd2: `pip install tkinterdnd2`
- Fallback to file browser if unavailable

**Issue**: High DPI display problems
- Windows: Add DPI awareness manifest
- Or scale the window: `self.root.geometry("600x750")`

**Issue**: Import errors
- Ensure magika is installed: `pip install magika`
- Check Python path includes installation directory

## Contributing

When contributing to the GUI:

1. Follow existing code style and patterns
2. Add docstrings to new methods
3. Update this documentation for new features
4. Test on multiple Windows versions (10, 11)
5. Ensure compatibility with Python 3.8+
6. Keep dependencies minimal

## Future Roadmap

Potential enhancements:

- [ ] Settings dialog for preferences
- [ ] Keyboard shortcuts
- [ ] Batch processing progress bar
- [ ] Export results to file
- [ ] Recent files history
- [ ] System tray integration
- [ ] Custom icon support
- [ ] Theme customization
- [ ] Multi-language support
- [ ] Context menu integration (right-click file)
- [ ] Drag-out file identification

## Resources

- [Tkinter Documentation](https://docs.python.org/3/library/tkinter.html)
- [tkinterdnd2 Documentation](https://github.com/pmgagne/tkinterdnd2)
- [PyInstaller Manual](https://pyinstaller.org/en/stable/)
- [Magika Documentation](https://securityresearch.google/magika/)

## License

All GUI code is licensed under Apache 2.0, matching the main Magika project.
