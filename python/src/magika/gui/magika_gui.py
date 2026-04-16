#!/usr/bin/env python3
# Copyright 2024 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Magika Windows GUI - A lightweight floating window for file type detection."""

import sys
import tkinter as tk
from tkinter import ttk, messagebox
from pathlib import Path
from typing import Optional
import threading

try:
    from magika import Magika
    from magika.types import PredictionMode
except ImportError:
    print("Error: Magika package not found. Please install with: pip install magika")
    sys.exit(1)


class MagikaGUI:
    """Main GUI class for Magika Windows interface."""

    def __init__(self, root: tk.Tk) -> None:
        """Initialize the Magika GUI.

        Args:
            root: The root Tkinter window.
        """
        self.root = root
        self.magika: Optional[Magika] = None
        self.is_processing = False

        # Window setup
        self.setup_window()

        # Initialize Magika in background
        self.init_magika()

        # Create UI
        self.create_widgets()

        # Setup drag and drop
        self.setup_drag_and_drop()

        # Enable window dragging
        self.setup_window_dragging()

    def setup_window(self) -> None:
        """Configure the main window properties."""
        self.root.title("Magika File Detector")
        self.root.geometry("400x500")

        # Make window semi-transparent
        self.root.attributes('-alpha', 0.95)

        # Always on top
        self.root.attributes('-topmost', True)

        # Remove window decorations for modern look
        self.root.overrideredirect(True)

        # Set background color
        self.root.configure(bg='#1e1e1e')

    def setup_window_dragging(self) -> None:
        """Enable dragging the window by clicking and dragging."""
        self.drag_x = 0
        self.drag_y = 0

        def start_drag(event):
            self.drag_x = event.x
            self.drag_y = event.y

        def on_drag(event):
            x = self.root.winfo_x() + event.x - self.drag_x
            y = self.root.winfo_y() + event.y - self.drag_y
            self.root.geometry(f"+{x}+{y}")

        # Bind to header frame
        self.root.bind('<Button-1>', start_drag)
        self.root.bind('<B1-Motion>', on_drag)

    def init_magika(self) -> None:
        """Initialize Magika instance in background thread."""
        def load_model():
            try:
                self.magika = Magika(
                    prediction_mode=PredictionMode.HIGH_CONFIDENCE
                )
                self.root.after(0, self.update_status, "Ready", "#4CAF50")
            except Exception as e:
                self.root.after(0, self.update_status,
                              f"Error: {str(e)}", "#f44336")

        threading.Thread(target=load_model, daemon=True).start()

    def create_widgets(self) -> None:
        """Create all UI widgets."""
        # Header frame
        header_frame = tk.Frame(self.root, bg='#2d2d2d', height=40)
        header_frame.pack(fill=tk.X)
        header_frame.pack_propagate(False)

        # Title
        title_label = tk.Label(
            header_frame,
            text="Magika",
            font=('Segoe UI', 14, 'bold'),
            fg='#ffffff',
            bg='#2d2d2d'
        )
        title_label.pack(side=tk.LEFT, padx=15, pady=8)

        # Close button
        close_btn = tk.Label(
            header_frame,
            text="×",
            font=('Segoe UI', 20),
            fg='#ffffff',
            bg='#2d2d2d',
            cursor='hand2'
        )
        close_btn.pack(side=tk.RIGHT, padx=10, pady=5)
        close_btn.bind('<Button-1>', lambda e: self.root.quit())
        close_btn.bind('<Enter>', lambda e: close_btn.config(bg='#f44336'))
        close_btn.bind('<Leave>', lambda e: close_btn.config(bg='#2d2d2d'))

        # Minimize button
        min_btn = tk.Label(
            header_frame,
            text="−",
            font=('Segoe UI', 20),
            fg='#ffffff',
            bg='#2d2d2d',
            cursor='hand2'
        )
        min_btn.pack(side=tk.RIGHT, padx=5, pady=5)
        min_btn.bind('<Button-1>', lambda e: self.root.iconify())
        min_btn.bind('<Enter>', lambda e: min_btn.config(bg='#424242'))
        min_btn.bind('<Leave>', lambda e: min_btn.config(bg='#2d2d2d'))

        # Main content frame
        content_frame = tk.Frame(self.root, bg='#1e1e1e')
        content_frame.pack(fill=tk.BOTH, expand=True, padx=20, pady=20)

        # Drop zone
        self.drop_zone = tk.Frame(
            content_frame,
            bg='#2d2d2d',
            relief=tk.FLAT,
            borderwidth=2
        )
        self.drop_zone.pack(fill=tk.BOTH, expand=True)

        drop_label = tk.Label(
            self.drop_zone,
            text="📁\n\nDrop files here\nor click to browse",
            font=('Segoe UI', 12),
            fg='#888888',
            bg='#2d2d2d',
            cursor='hand2'
        )
        drop_label.pack(expand=True)
        drop_label.bind('<Button-1>', self.browse_file)

        # Results frame with scrollbar
        results_container = tk.Frame(content_frame, bg='#1e1e1e')
        results_container.pack(fill=tk.BOTH, expand=True, pady=(10, 0))

        # Scrollbar
        scrollbar = tk.Scrollbar(results_container)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)

        # Results text widget
        self.results_text = tk.Text(
            results_container,
            font=('Consolas', 9),
            fg='#e0e0e0',
            bg='#252525',
            relief=tk.FLAT,
            wrap=tk.WORD,
            yscrollcommand=scrollbar.set,
            state=tk.DISABLED
        )
        self.results_text.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.config(command=self.results_text.yview)

        # Status bar
        status_frame = tk.Frame(self.root, bg='#2d2d2d', height=30)
        status_frame.pack(fill=tk.X, side=tk.BOTTOM)
        status_frame.pack_propagate(False)

        self.status_label = tk.Label(
            status_frame,
            text="Loading model...",
            font=('Segoe UI', 9),
            fg='#FFC107',
            bg='#2d2d2d',
            anchor=tk.W
        )
        self.status_label.pack(fill=tk.X, padx=10, pady=5)

    def setup_drag_and_drop(self) -> None:
        """Setup drag and drop functionality for files."""
        try:
            from tkinterdnd2 import DND_FILES, TkinterDnD
            # If tkinterdnd2 is available, upgrade the window
            # This requires reinstalling with: pip install tkinterdnd2
            pass
        except ImportError:
            # Fallback: just use file dialog
            pass

    def browse_file(self, event=None) -> None:
        """Open file browser dialog."""
        from tkinter import filedialog

        files = filedialog.askopenfilenames(
            title="Select files to analyze",
            parent=self.root
        )

        if files:
            self.analyze_files(files)

    def analyze_files(self, file_paths) -> None:
        """Analyze the selected files.

        Args:
            file_paths: List of file paths to analyze.
        """
        if self.is_processing:
            return

        if not self.magika:
            messagebox.showerror(
                "Error",
                "Magika is not initialized yet. Please wait.",
                parent=self.root
            )
            return

        self.is_processing = True
        self.update_status("Analyzing...", "#2196F3")

        def analyze():
            try:
                results = []
                for file_path in file_paths:
                    path = Path(file_path)
                    result = self.magika.identify_path(path)

                    results.append({
                        'path': path.name,
                        'type': result.output.description,
                        'label': result.output.label,
                        'mime': result.output.mime_type,
                        'score': result.score
                    })

                self.root.after(0, self.display_results, results)
                self.root.after(0, self.update_status, "Complete", "#4CAF50")
            except Exception as e:
                self.root.after(0, messagebox.showerror,
                              "Error", str(e), self.root)
                self.root.after(0, self.update_status,
                              "Error occurred", "#f44336")
            finally:
                self.is_processing = False

        threading.Thread(target=analyze, daemon=True).start()

    def display_results(self, results) -> None:
        """Display analysis results in the text widget.

        Args:
            results: List of result dictionaries.
        """
        self.results_text.config(state=tk.NORMAL)
        self.results_text.delete(1.0, tk.END)

        for i, result in enumerate(results):
            if i > 0:
                self.results_text.insert(tk.END, "\n" + "─" * 50 + "\n\n")

            self.results_text.insert(tk.END, f"📄 {result['path']}\n", 'filename')
            self.results_text.insert(tk.END, f"\nType: {result['type']}\n")
            self.results_text.insert(tk.END, f"Label: {result['label']}\n")
            self.results_text.insert(tk.END, f"MIME: {result['mime']}\n")
            self.results_text.insert(tk.END,
                                    f"Confidence: {result['score']:.1%}\n")

        # Configure tags for styling
        self.results_text.tag_config('filename',
                                     font=('Consolas', 10, 'bold'),
                                     foreground='#64B5F6')

        self.results_text.config(state=tk.DISABLED)

    def update_status(self, message: str, color: str) -> None:
        """Update the status bar message.

        Args:
            message: Status message to display.
            color: Color for the status text.
        """
        self.status_label.config(text=message, fg=color)


def main() -> None:
    """Main entry point for the GUI application."""
    try:
        # Try to use TkinterDnD for drag-and-drop support
        from tkinterdnd2 import TkinterDnD
        root = TkinterDnD.Tk()
    except ImportError:
        # Fallback to regular Tkinter
        root = tk.Tk()

    app = MagikaGUI(root)
    root.mainloop()


if __name__ == "__main__":
    main()
