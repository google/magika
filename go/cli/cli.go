package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"math"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/google/magika/go/magika"
)

const (
	// Env fallbacks for --assets-dir and --model. Flags take precedence.
	assetsDirEnv = "MAGIKA_ASSETS_DIR"
	modelNameEnv = "MAGIKA_MODEL"

	// ANSI escape sequences used for coloring output.
	ansiReset = "\x1b[0m"
	ansiBold  = "\x1b[1m"
)

// errHadErrors signals at least one path failed to scan.
var errHadErrors = errors.New("one or more files failed to scan")

// cliFlags parsed command-line configuration.
type cliFlags struct {
	// Traversal.
	recursive     bool // -r, --recursive: descend into directories.
	noDereference bool // --no-dereference: stat symlinks instead of following them.

	// Colors.
	colors   bool // --colors: force ANSI colors even when stdout is not a TTY.
	noColors bool // --no-colors: disable colors even when stdout is a TTY.

	// Output modifiers (used only by the default text format).
	outputScore bool // -s, --output-score: append the score to each line.
	mimeType    bool // -i, --mime-type: print the MIME type instead of the description.
	label       bool // -l, --label: print the short label instead of the description.

	// Output format (mutually exclusive with each other).
	json   bool   // --json: emit a pretty-printed JSON array.
	jsonl  bool   // --jsonl: emit one JSON object per line.
	format string // --format: custom template with %p %l %d %g %m %e %s %S %b %%.

	// Model selection.
	assetsDir string // --assets-dir: overrides $MAGIKA_ASSETS_DIR.
	modelName string // --model:      overrides $MAGIKA_MODEL.
}

// outcome classification result for a single path. dl is the raw model
// prediction before low-confidence / Overwrite remaps; ct is the final
// type after those remaps. For ruled outcomes (directory, symlink) the
// model never ran and dl is zero.
type outcome struct {
	path  string
	dl    magika.ContentType
	ct    magika.ContentType
	score float32
	ruled bool
	err   error
}

// jsonTypeInfo mirrors the "dl" and "output" objects of the JSON output.
type jsonTypeInfo struct {
	Description string   `json:"description"`
	Extensions  []string `json:"extensions"`
	Group       string   `json:"group"`
	IsText      bool     `json:"is_text"`
	Label       string   `json:"label"`
	MimeType    string   `json:"mime_type"`
}

// jsonOK is the "value" payload emitted when status == "ok".
type jsonOK struct {
	Dl     jsonTypeInfo `json:"dl"`
	Output jsonTypeInfo `json:"output"`
	Score  float32      `json:"score"`
}

// jsonResult is the "result" object inside each entry. Value is omitted on
// failure so only the status is rendered (e.g. {"status":"permission_error"}).
type jsonResult struct {
	Status string `json:"status"`
	Value  any    `json:"value,omitempty"`
}

// jsonTopLevel is one entry of the top-level array (for --json) or one line of
// --jsonl output.
type jsonTopLevel struct {
	Path   string     `json:"path"`
	Result jsonResult `json:"result"`
}

// parseArgs parses args into cliFlags and positional paths.
func parseArgs(stderr io.Writer, args []string) (cliFlags, []string, error) {
	fs := flag.NewFlagSet("magika", flag.ContinueOnError)
	fs.SetOutput(stderr)
	var f cliFlags
	fs.BoolVar(&f.recursive, "recursive", false, "Identify files within directories instead of the directory itself.")
	fs.BoolVar(&f.recursive, "r", false, "Alias for --recursive.")
	fs.BoolVar(&f.noDereference, "no-dereference", false, "Identify symbolic links as is instead of following them.")
	fs.BoolVar(&f.colors, "colors", false, "Print with colors regardless of terminal support.")
	fs.BoolVar(&f.noColors, "no-colors", false, "Print without colors regardless of terminal support.")
	fs.BoolVar(&f.outputScore, "output-score", false, "Print the prediction score in addition to the content type.")
	fs.BoolVar(&f.outputScore, "s", false, "Alias for --output-score.")
	fs.BoolVar(&f.mimeType, "mime-type", false, "Print the MIME type instead of the description.")
	fs.BoolVar(&f.mimeType, "i", false, "Alias for --mime-type.")
	fs.BoolVar(&f.label, "label", false, "Print a simple label instead of the description.")
	fs.BoolVar(&f.label, "l", false, "Alias for --label.")
	fs.BoolVar(&f.json, "json", false, "Print results as a JSON array.")
	fs.BoolVar(&f.jsonl, "jsonl", false, "Print results as JSONL (one JSON object per line).")
	fs.StringVar(&f.format, "format", "", "Custom format with placeholders %p %l %d %g %m %e %s %S %b %%.")
	fs.StringVar(&f.assetsDir, "assets-dir", "", "Path to the magika assets directory (overrides $"+assetsDirEnv+").")
	fs.StringVar(&f.modelName, "model", "", "Model name to load from the assets dir (overrides $"+modelNameEnv+").")
	fs.Usage = func() {
		fmt.Fprintf(stderr, "Usage: magika [OPTIONS] [PATH]...\n\n")
		fs.PrintDefaults()
	}
	if err := fs.Parse(args); err != nil {
		return cliFlags{}, nil, err
	}
	if f.colors && f.noColors {
		return cliFlags{}, nil, errors.New("--colors and --no-colors are mutually exclusive")
	}
	modifiersSelected := 0
	if f.mimeType {
		modifiersSelected++
	}
	if f.label {
		modifiersSelected++
	}
	if modifiersSelected > 1 {
		return cliFlags{}, nil, errors.New("--mime-type and --label are mutually exclusive")
	}
	formatSelected := 0
	if f.json {
		formatSelected++
	}
	if f.jsonl {
		formatSelected++
	}
	if f.format != "" {
		formatSelected++
	}
	if formatSelected > 1 {
		return cliFlags{}, nil, errors.New("--json, --jsonl and --format are mutually exclusive")
	}
	paths := fs.Args()
	if len(paths) == 0 {
		fs.Usage()
		return cliFlags{}, nil, errors.New("no paths given")
	}
	dashes := 0
	for _, p := range paths {
		if p == "-" {
			dashes++
		}
	}
	if dashes > 1 {
		return cliFlags{}, nil, errors.New("only one path can be the standard input")
	}
	return f, paths, nil
}

// cli entry point.
func cli(stdout io.Writer, stderr io.Writer, stdin io.Reader, args ...string) error {
	f, paths, err := parseArgs(stderr, args)
	if err != nil {
		return err
	}

	assetsDir := f.assetsDir
	if assetsDir == "" {
		assetsDir = os.Getenv(assetsDirEnv)
	}
	if assetsDir == "" {
		return fmt.Errorf("assets dir not set: pass --assets-dir or set %s", assetsDirEnv)
	}
	modelName := f.modelName
	if modelName == "" {
		modelName = os.Getenv(modelNameEnv)
	}
	if modelName == "" {
		return fmt.Errorf("model name not set: pass --model or set %s", modelNameEnv)
	}
	s, err := magika.NewScanner(assetsDir, modelName)
	if err != nil {
		return fmt.Errorf("create scanner: %w", err)
	}

	useColor := pickColor(stdout, f.colors, f.noColors)

	// BFS-like queue: recursive directory traversal prepends children so the
	// original argument order is preserved in the output.
	queue := make([]string, 0, len(paths))
	queue = append(queue, paths...)
	var results []*outcome
	hadError := false

	for len(queue) > 0 {
		path := queue[0]
		queue = queue[1:]
		oc, recursed := classify(s, stdin, path, f, &queue)
		if recursed {
			continue
		}
		results = append(results, oc)
		if oc.err != nil {
			hadError = true
		}
	}

	switch {
	case f.json:
		if err := emitJSONArray(stdout, results); err != nil {
			return err
		}
	case f.jsonl:
		for _, oc := range results {
			b, err := json.Marshal(jsonEntry(oc))
			if err != nil {
				return err
			}
			fmt.Fprintln(stdout, string(b))
		}
	default:
		for _, oc := range results {
			fmt.Fprintln(stdout, renderText(oc, f, useColor))
		}
	}

	if hadError {
		return errHadErrors
	}
	return nil
}

// classify produces an outcome for a single path. Handles "-" (stdin),
// directories (ruled, or recurse by prepending sorted children and returning
// (nil, true)), and symlinks under --no-dereference. I/O or scan errors are
// recorded on the outcome so one bad path does not abort the batch.
func classify(s *magika.Scanner, stdin io.Reader, path string, f cliFlags, queue *[]string) (*outcome, bool) {
	if path == "-" {
		data, err := io.ReadAll(stdin)
		if err != nil {
			return &outcome{path: path, err: err}, false
		}
		dl, ct, score, err := s.ScanDetails(bytes.NewReader(data), len(data))
		return &outcome{path: path, dl: dl, ct: ct, score: score, err: err}, false
	}

	var info os.FileInfo
	var err error
	if f.noDereference {
		info, err = os.Lstat(path)
	} else {
		info, err = os.Stat(path)
	}
	if err != nil {
		return &outcome{path: path, err: err}, false
	}

	if info.IsDir() {
		if !f.recursive {
			return &outcome{path: path, ct: directoryContentType, score: 1, ruled: true}, false
		}
		entries, err := os.ReadDir(path)
		if err != nil {
			return &outcome{path: path, err: err}, false
		}
		names := make([]string, 0, len(entries))
		for _, e := range entries {
			names = append(names, filepath.Join(path, e.Name()))
		}
		sort.Strings(names)
		*queue = append(names, *queue...)
		return nil, true
	}
	if info.Mode()&os.ModeSymlink != 0 {
		return &outcome{path: path, ct: symlinkContentType, score: 1, ruled: true}, false
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return &outcome{path: path, err: err}, false
	}
	dl, ct, score, err := s.ScanDetails(bytes.NewReader(data), len(data))
	return &outcome{path: path, dl: dl, ct: ct, score: score, err: err}, false
}

// Synthetic content types the CLI injects for cases the scanner never sees.
// Mirror content_types_kb.min.json file.
var directoryContentType = magika.ContentType{
	Label:       "directory",
	MimeType:    "inode/directory",
	Group:       "inode",
	Description: "A directory",
	Extensions:  []string{},
	IsText:      false,
}

var symlinkContentType = magika.ContentType{
	Label:       "symlink",
	MimeType:    "inode/symlink",
	Group:       "inode",
	Description: "Symbolic link",
	Extensions:  []string{},
	IsText:      false,
}

var undefinedContentType = magika.ContentType{
	Label:       "undefined",
	MimeType:    "application/octet-stream",
	Group:       "undefined",
	Description: "Undefined",
	Extensions:  []string{},
	IsText:      false,
}

// pickColor decides whether ANSI colors should be emitted.
func pickColor(w io.Writer, enable, disable bool) bool {
	if disable {
		return false
	}
	if enable {
		return true
	}
	file, ok := w.(*os.File)
	if !ok {
		return false
	}
	info, err := file.Stat()
	if err != nil {
		return false
	}
	return info.Mode()&os.ModeCharDevice != 0
}

// renderText formats an outcome as a single line.
func renderText(oc *outcome, f cliFlags, color bool) string {
	format := f.format
	if format == "" {
		sb := strings.Builder{}
		sb.WriteString("%p: ")
		switch {
		case f.mimeType:
			sb.WriteString("%m")
		case f.label:
			sb.WriteString("%l")
		default:
			sb.WriteString("%d (%g)")
		}
		if f.outputScore {
			sb.WriteString(" %S")
		}
		format = sb.String()
	}
	out := expandFormat(format, oc)
	if color {
		return colorize(oc, out)
	}
	return out
}

// expandFormat substitutes %-placeholders with outcome fields.
//
// Supported placeholders:
//
//		%p  path
//		%l  label
//		%d  description
//		%g  group
//		%m  MIME type
//		%e  [ext1,ext2,...] extensions
//		%s  score as "0.42"
//		%S  score as integer percent, e.g. "42%"
//		%b  model best-guess on low confidence (always empty here, see below)
//		%%  literal '%'
//
//	 %b emits the raw DL prediction label (empty for ruled outcomes where the
//	 model did not run).
//
//	 Unknown placeholders are emitted verbatim, a trailing
//	 '%' is dropped.
func expandFormat(format string, oc *outcome) string {
	var out strings.Builder
	runes := []rune(format)
	for i := 0; i < len(runes); i++ {
		if runes[i] != '%' {
			out.WriteRune(runes[i])
			continue
		}
		i++
		if i >= len(runes) {
			break
		}
		switch runes[i] {
		case 'p':
			out.WriteString(oc.path)
		case 'l':
			out.WriteString(oc.labelField())
		case 'd':
			out.WriteString(oc.descriptionField())
		case 'g':
			out.WriteString(oc.groupField())
		case 'm':
			out.WriteString(oc.mimeTypeField())
		case 'e':
			out.WriteString(formatExtensions(oc.extensionsField()))
		case 's':
			fmt.Fprintf(&out, "%.2f", oc.scoreField())
		case 'S':
			pct := math.Trunc(float64(oc.scoreField()) * 100)
			fmt.Fprintf(&out, "%d%%", int(pct))
		case 'b':
			if oc.err == nil && !oc.ruled {
				out.WriteString(oc.dl.Label)
			}
		case '%':
			out.WriteByte('%')
		default:
			out.WriteRune(runes[i])
		}
	}
	return out.String()
}

// formatExtensions renders an extension list in the compact bracket form used
// by the %e placeholder, e.g. []string{"py","pyi"} -> "[py,pyi]".
func formatExtensions(xs []string) string {
	return "[" + strings.Join(xs, ",") + "]"
}

// labelField, descriptionField, groupField, mimeTypeField, extensionsField,
// and scoreField return the per-field values used by expandFormat.
// On error the fallbacks are: literal "error" for label / group / mime, the error
// message for description, nil extensions, and a score of 1 to mean "no
// confidence applies".
func (o *outcome) labelField() string {
	if o.err != nil {
		return "error"
	}
	return o.ct.Label
}

func (o *outcome) descriptionField() string {
	if o.err != nil {
		return o.err.Error()
	}
	return o.ct.Description
}

func (o *outcome) groupField() string {
	if o.err != nil {
		return "error"
	}
	return o.ct.Group
}

func (o *outcome) mimeTypeField() string {
	if o.err != nil {
		return "error"
	}
	return o.ct.MimeType
}

func (o *outcome) extensionsField() []string {
	if o.err != nil {
		return nil
	}
	return o.ct.Extensions
}

func (o *outcome) scoreField() float32 {
	if o.err != nil {
		return 1
	}
	return o.score
}

// trueColor returns the SGR escape that sets the foreground to a 24-bit RGB
// color. Terminals without trueColor support typically downgrade it to the
// nearest 8/16-color equivalent.
func trueColor(r, g, b int) string {
	return fmt.Sprintf("\x1b[38;2;%d;%d;%dm", r, g, b)
}

// colorize wraps text in an ANSI color matching the outcome's group.
// Errors render bold red; other groups use the Tailwind 500 palette.
func colorize(oc *outcome, text string) string {
	if oc.err != nil {
		return ansiBold + "\x1b[31m" + text + ansiReset
	}
	var prefix string
	switch oc.ct.Group {
	case "application":
		prefix = trueColor(0xf4, 0x3f, 0x5e) // Rose 500
	case "archive":
		prefix = trueColor(0xf5, 0x9e, 0x0b) // Amber 500
	case "audio":
		prefix = trueColor(0x84, 0xcc, 0x16) // Lime 500
	case "code":
		prefix = trueColor(0x8b, 0x5c, 0xf6) // Violet 500
	case "document":
		prefix = trueColor(0x3b, 0x82, 0xf6) // Blue 500
	case "executable":
		prefix = trueColor(0xec, 0x48, 0x99) // Pink 500
	case "image":
		prefix = trueColor(0x06, 0xb6, 0xd4) // Cyan 500
	case "video":
		prefix = trueColor(0x10, 0xb9, 0x81) // Emerald 500
	default:
		prefix = ansiBold + trueColor(0xcc, 0xcc, 0xcc)
	}
	return prefix + text + ansiReset
}

// typeInfo converts a magika.ContentType into the JSON-serialisable form and
// replaces a nil Extensions slice with []string{} so the field never marshals
// to null.
func typeInfo(ct magika.ContentType) jsonTypeInfo {
	ext := ct.Extensions
	if ext == nil {
		ext = []string{}
	}
	return jsonTypeInfo{
		Description: ct.Description,
		Extensions:  ext,
		Group:       ct.Group,
		IsText:      ct.IsText,
		Label:       ct.Label,
		MimeType:    ct.MimeType,
	}
}

// jsonEntry converts one outcome into its JSON representation.
// For ruled outcomes (directory, symlink) dl is "undefined" since the model never ran.
// The score is truncated to three decimals so output stays stable.
func jsonEntry(oc *outcome) jsonTopLevel {
	entry := jsonTopLevel{Path: oc.path}
	if oc.err != nil {
		entry.Result = jsonResult{Status: jsonErrorKind(oc.err)}
		return entry
	}
	dl := typeInfo(oc.dl)
	if oc.ruled {
		dl = typeInfo(undefinedContentType)
	}
	score := float32(math.Trunc(float64(oc.score)*1000) / 1000)
	entry.Result = jsonResult{
		Status: "ok",
		Value: jsonOK{
			Dl:     dl,
			Output: typeInfo(oc.ct),
			Score:  score,
		},
	}
	return entry
}

// jsonErrorKind maps a scan error to one of the emitted status tags:
// "file_does_not_exist", "permission_error", or "unknown" for anything
// else (read errors, I/O errors, scan failures, ...).
func jsonErrorKind(err error) string {
	switch {
	case errors.Is(err, os.ErrNotExist):
		return "file_does_not_exist"
	case errors.Is(err, os.ErrPermission):
		return "permission_error"
	default:
		return "unknown"
	}
}

// emitJSONArray writes the outcomes as a pretty-printed JSON array.
// The formatting (two-space indent, leading newline after '[') is produced
// manually so the layout is stable regardless of json.Encoder defaults.
func emitJSONArray(w io.Writer, outcomes []*outcome) error {
	if len(outcomes) == 0 {
		_, err := fmt.Fprintln(w, "[]")
		return err
	}
	if _, err := fmt.Fprint(w, "["); err != nil {
		return err
	}
	for i, oc := range outcomes {
		if i > 0 {
			if _, err := fmt.Fprint(w, ","); err != nil {
				return err
			}
		}
		b, err := json.MarshalIndent(jsonEntry(oc), "  ", "  ")
		if err != nil {
			return err
		}
		if _, err := fmt.Fprint(w, "\n  ", string(b)); err != nil {
			return err
		}
	}
	_, err := fmt.Fprintln(w, "\n]")
	return err
}
