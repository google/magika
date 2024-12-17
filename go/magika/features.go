package magika

import (
	"bytes"
	"fmt"
	"io"
)

// Features holds the features of a give slice of bytes.
type Features struct {
	firstBlock []byte
	Beg        []int32 `json:"beg"`
	Mid        []int32 `json:"mid"`
	End        []int32 `json:"end"`
	Offset8000 []int32 `json:"offset_0x8000_0x8007"`
	Offset8800 []int32 `json:"offset_0x8800_0x8807"`
	Offset9000 []int32 `json:"offset_0x9000_0x9007"`
	Offset9800 []int32 `json:"offset_0x9800_0x9807"`
}

// ExtractFeatures extract the features from the given reader.
// The number of bytes that can be read from the reader is given by size.
func ExtractFeatures(cfg Config, r io.ReaderAt, size int) (Features, error) {
	var (
		er  = errReader{r: r, sz: size}
		beg = er.readAt(0, cfg.BlockSize)
		mid = er.readAt((size-cfg.MidSize)/2, cfg.MidSize)
		end = er.readAt(size-cfg.BlockSize, cfg.BlockSize)
	)
	f := buildFeatures(cfg, beg, mid, end)

	peek := func(off int) []int32 {
		b := er.readAt(off, 8)
		if len(b) < 8 {
			b = nil
		}
		return padInt32(cfg, b, 0, 8)
	}
	f.Offset8000 = peek(0x8000)
	f.Offset8800 = peek(0x8800)
	f.Offset9000 = peek(0x9000)
	f.Offset9800 = peek(0x9800)

	if er.err != nil {
		return Features{}, er.err
	}
	return f, nil
}

// Flatten returns a flattened array of the given features.
func (f Features) Flatten() []int32 {
	res := make([]int32, 0, len(f.Beg)+len(f.Mid)+len(f.End))
	res = append(res, f.Beg...)
	res = append(res, f.Mid...)
	res = append(res, f.End...)
	return res
}

// errReader wraps an io.ReaderAt and accumulates errors that may arise during
// reading. It also silently protects against out of range read.
// This allows for a simpler parsing code flow with a unique error check at
// the end of parsing.
type errReader struct {
	r   io.ReaderAt
	sz  int
	err error
}

func (e *errReader) readAt(off, n int) []byte {
	if e.err != nil || off >= e.sz {
		return nil
	}
	if off < 0 {
		n += off
		off = 0
	}
	n = min(n, e.sz-off)
	b := make([]byte, n)
	p, err := e.r.ReadAt(b, int64(max(off, 0)))
	if err != nil && err != io.EOF {
		e.err = fmt.Errorf("read %d bytes at %d: %w", n, max(off, 0), err)
		return nil
	}
	return b[:p]
}

// buildFeatures builds features from the beg, mid, and end bytes.
func buildFeatures(cfg Config, beg, mid, end []byte) Features {
	firstBlock := beg

	spaces := string([]rune{'\t', '\n', '\v', '\f', '\r', ' '})
	// Trim beg and end, and truncate to BegSize and EndSize.
	beg = bytes.TrimLeft(beg, spaces)
	end = bytes.TrimRight(end, spaces)
	beg = safeSlice(beg, 0, cfg.BegSize)
	end = safeSlice(end, len(end)-cfg.EndSize, len(end))

	return Features{
		firstBlock: firstBlock,
		Beg:        padInt32(cfg, beg, 0, cfg.BegSize),
		Mid:        padInt32(cfg, mid, (cfg.MidSize-len(mid))/2, cfg.MidSize),
		End:        padInt32(cfg, end, cfg.EndSize-len(end), cfg.EndSize),
	}
}

// padInt32 pads and convert the given bytes into int32.
// The len of the returned is the given size.
// if prefix is non-zero, that many padding is add as prefix.
// then the given bytes are converted into int32
// finally, padding occurs until the returned slice is of the given size.
func padInt32(cfg Config, b []byte, prefix, size int) []int32 {
	r := make([]int32, 0, size)
	for len(r) < prefix {
		r = append(r, int32(cfg.PaddingToken))
	}
	for _, bb := range b {
		r = append(r, int32(bb))
	}
	for len(r) < size {
		r = append(r, int32(cfg.PaddingToken))
	}
	return r
}

// safeSlice returns a slice from the given array, silently clipping
// out-of-bound indices. This happens when the given input data contains
// fewer bytes than the sampling size.
func safeSlice(b []byte, from, to int) []byte {
	return b[max(from, 0):min(to, len(b))]
}
