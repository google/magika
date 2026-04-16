# Magika GUI Quick Start

Get started with the Magika Windows GUI in under 2 minutes!

## Installation (30 seconds)

```bash
pip install magika tkinterdnd2
```

## Launch (5 seconds)

```bash
python -m magika.gui
```

Or on Windows:
```bash
pythonw -m magika.gui
```

## First Analysis (30 seconds)

1. **Window appears** - A semi-transparent floating window will appear on your screen
2. **Drop a file** - Drag any file from your desktop or file explorer onto the window
3. **View results** - File type, MIME type, and confidence score appear instantly

## Example Output

```
📄 document.pdf

Type: PDF document
Label: pdf
MIME: application/pdf
Confidence: 99.8%
Path: C:\Users\You\Desktop\document.pdf
```

## Common Use Cases

### Identify Unknown Files
Drop any file with an unknown or missing extension to instantly identify its true type.

### Verify File Types
Ensure downloaded files are what they claim to be by checking their true content type.

### Batch Analysis
Select multiple files at once to analyze them all in a single operation.

### Quick Access
Keep the window open and always-on-top for instant file identification throughout your workday.

## Keyboard Shortcuts

- **Drag window**: Click and drag the title bar
- **Minimize**: Click the "−" button (or minimize normally)
- **Close**: Click the "×" button (or Alt+F4)

## Tips

1. **Pin to Taskbar**: Right-click the window in taskbar → Pin to taskbar
2. **Startup Shortcut**: Add to your Startup folder for automatic launch
3. **Multiple Files**: Ctrl+Click in the file browser to select multiple files
4. **Transparency**: Edit `magika_gui.py` to adjust window transparency

## Troubleshooting

**Window won't appear?**
- Check Python has GUI support: `python -m tkinter`

**Drag-and-drop not working?**
- Click the drop zone to use file browser instead

**Window too small/large?**
- Edit window size in `magika_gui.py` (search for `geometry`)

## Next Steps

- Read the [full documentation](GUI_WINDOWS.md) for advanced features
- Check out the [development guide](GUI_DEVELOPMENT.md) to customize the GUI
- Create a desktop shortcut for easy access

## That's it!

You're now ready to identify file types with Magika's GUI. Enjoy! 🎉
