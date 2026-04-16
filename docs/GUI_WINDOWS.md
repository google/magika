# Magika Windows GUI

A lightweight, semi-transparent floating window interface for Magika file type detection on Windows.

## Features

- **Modern UI**: Clean, minimalist design with semi-transparent window
- **Floating Window**: Always-on-top, draggable window that stays accessible
- **Drag & Drop**: Drop files directly onto the window for instant analysis
- **File Browser**: Click the drop zone to browse and select files
- **Detailed Results**: Shows file type, MIME type, label, and confidence score
- **Multi-file Support**: Analyze multiple files at once
- **Responsive**: Non-blocking UI with background processing

## Installation

### Prerequisites

- Python 3.8 or higher
- Windows OS
- Magika package installed

### Install Magika with GUI support

```bash
pip install magika tkinterdnd2
```

Note: `tkinterdnd2` is optional but recommended for drag-and-drop functionality.

### Alternative: Install from source

```bash
git clone https://github.com/google/magika.git
cd magika/python
pip install -e .
pip install tkinterdnd2
```

## Usage

### Launch the GUI

```bash
python -m magika.gui
```

Or use the launcher script:

```bash
python magika-gui.py
```

### Creating a Desktop Shortcut (Windows)

1. Right-click on your desktop and select **New > Shortcut**
2. Enter the following as the location:
   ```
   pythonw.exe -m magika.gui
   ```
   (or use the full path to `magika-gui.py`)
3. Name the shortcut "Magika File Detector"
4. Click Finish

You can also assign a custom icon to the shortcut.

### Using the GUI

1. **Drop Files**: Drag and drop one or more files onto the window
2. **Browse Files**: Click the drop zone to open a file browser
3. **View Results**: Analysis results appear in the results panel with:
   - File name
   - Content type description
   - Label
   - MIME type
   - Confidence score
4. **Move Window**: Click and drag the title bar to reposition
5. **Minimize**: Click the "−" button in the top-right
6. **Close**: Click the "×" button in the top-right

## UI Overview

```
┌─────────────────────────────┐
│ Magika              − × │  ← Title bar (drag to move)
├─────────────────────────────┤
│                             │
│   📁                        │
│                             │
│   Drop files here           │  ← Drop zone (click to browse)
│   or click to browse        │
│                             │
├─────────────────────────────┤
│ 📄 example.pdf              │
│                             │
│ Type: PDF document          │  ← Results panel
│ Label: pdf                  │
│ MIME: application/pdf       │
│ Confidence: 99.8%           │
├─────────────────────────────┤
│ Ready                       │  ← Status bar
└─────────────────────────────┘
```

## Configuration

### Transparency Level

To adjust window transparency, edit `magika_gui.py`:

```python
self.root.attributes('-alpha', 0.95)  # 0.0 (transparent) to 1.0 (opaque)
```

### Window Size

Default size is 400x500. To change:

```python
self.root.geometry("400x500")  # Change to your preferred size
```

### Prediction Mode

The GUI uses `HIGH_CONFIDENCE` mode by default. To change:

```python
self.magika = Magika(
    prediction_mode=PredictionMode.MEDIUM_CONFIDENCE  # or BEST_GUESS
)
```

## Troubleshooting

### Drag-and-drop not working

If drag-and-drop doesn't work, make sure `tkinterdnd2` is installed:

```bash
pip install tkinterdnd2
```

If the issue persists, you can still use the file browser by clicking the drop zone.

### Window appears behind other windows

The window should stay on top by default. If it doesn't, try:
- Clicking on the window to bring it to focus
- Restarting the application

### High DPI displays

On high DPI displays, you may want to increase the window size:

```python
self.root.geometry("600x750")
```

### GUI doesn't start

Ensure all dependencies are installed:

```bash
pip install magika tkinterdnd2
```

Check Python version:

```bash
python --version  # Should be 3.8 or higher
```

## Building an Executable

To create a standalone `.exe` file using PyInstaller:

```bash
pip install pyinstaller
pyinstaller --onefile --windowed --name "Magika" magika-gui.py
```

The executable will be in the `dist/` folder.

## License

Apache 2.0 - See LICENSE file for details.

## Disclaimer

This is not an official Google product. It is not supported by Google and 
Google specifically disclaims all warranties as to its quality, 
merchantability, or fitness for a particular purpose.
