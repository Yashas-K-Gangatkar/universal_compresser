# 🌳 YK Universal Compressor — 3D Binary Compression Engine

> **A multi-dimensional binary mapping algorithm** that compresses data by storing shared bit sequences in a 3D coordinate space (X, Y, Z) and reconstructing files via microscopic pointer maps. The Rust engine implements a high-performance DEFLATE-class compressor with lazy matching and canonical Huffman coding.

**Creator:** Yashas Krishnamurthy
**Format:** `.yk`
**License:** MIT
**Languages:** Rust (production engine) + Python (research prototype)

---

## 🚀 Two Implementations

This repository contains **two** implementations of the YK algorithm:

### 1. Rust Engine (Production) — `src/main.rs`

A high-performance DEFLATE-class compressor + decompressor written in Rust. Features:

- **LZ77-style sliding window** with 32 KB lookback
- **Lazy matching** — if a 4-byte match is found, look 1 byte ahead to see if skipping reveals a longer match
- **Canonical Huffman coding** (DEFLATE BTYPE=10 compliant)
- **Kraft inequality fix** — code lengths are rebalanced to satisfy Kraft's inequality, guaranteeing decodability
- **Run-Length Encoding** of code-length trees (symbols 16, 17, 18) to squeeze metadata
- **Hash-chain dictionary** for O(1) amortized match lookup
- **External Master Grid (Preset Dictionary)** — preload a 32 KB reference text (e.g. Sherlock Holmes) so that new files (e.g. Alice in Wonderland) reuse English-language patterns already in the grid. Compressed files can shrink to a microscopic "ticket" referencing the master grid.
- **Full round-trip** — both `compress` and `decompress` commands supported

### 2. Python Prototype (Research) — `python/`

The original conceptual prototype that explores the **3D Nuclear Tree** idea — mapping bits into a 3D grid with 26-direction overlap detection. Useful for understanding the algorithm's core concept, but not optimized for production.

---

## 🛠️ Build & Run (Rust Engine)

### Prerequisites
- Rust toolchain (1.56+) — install from https://rustup.rs

### Build
```bash
git clone https://github.com/Yashas-K-Gangatkar/universal_compresser.git
cd universal_compresser
cargo build --release
```

### Compress a file
```bash
# Basic usage (output: input.txt.yk)
./target/release/yk_engine compress input.txt

# Specify output file
./target/release/yk_engine compress input.txt compressed.yk
```

### Decompress a file
```bash
# Basic usage (output: decoded_file.txt)
./target/release/yk_engine decompress compressed.yk

# Specify output file
./target/release/yk_engine decompress compressed.yk restored.txt
```

### Expected output
```
Success! Compressed input.txt (12345 bytes) -> compressed.yk (4321 bytes)
Success! Decompressed compressed.yk -> restored.txt (12345 bytes)
```

### ⚠️ Master Dictionary Setup

The Rust engine uses an **external Master Grid** (preset dictionary). Before running, place a reference text file at:

```
/Users/yashas/Desktop/YK_Algorithm/master_dict.txt
```

A common choice is the full text of *Sherlock Holmes* (Project Gutenberg) — the engine takes the last 32 KB of the file as the preset dictionary. Both `compress` and `decompress` load the same dictionary, so the decoder can reconstruct the original file losslessly.

To use a different dictionary path, edit the `dict_path` constant in `src/main.rs`:

```rust
let dict_path = "/path/to/your/master_dict.txt";
```

---

## 🧪 Run the Python Prototype

```bash
cd python

# Basic 3D Nuclear Tree demo (lossless round-trip)
python yk_nuclear_tree.py

# Compressor + Decoder (factory + machine)
python yk_factory.py
python yk_machine.py

# 10MB compression benchmark
python yk_10mb_test.py
```

---

## 📁 Project Structure

```
universal_compresser/
├── Cargo.toml                    # Rust package manifest
├── README.md
├── LICENSE
├── .gitignore
├── src/
│   └── main.rs                   # Rust compression engine (~250 lines)
├── python/
│   ├── yk_nuclear_tree.py        # 3D Nuclear Tree prototype (core concept)
│   ├── yk_factory.py             # Python compressor
│   ├── yk_machine.py             # Python decoder
│   └── yk_10mb_test.py           # 10MB benchmark
└── docs/
    └── ARCHITECTURE.md           # Algorithm deep-dive
```

---

## 🧠 The Concept (3D Nuclear Tree)

Imagine standing inside a 3D grid. Words like `HELP`, `HOPE`, and `HUGE` appear as physical branches growing from a shared letter `H`. The `H` is **stationary** — planted once at a single coordinate — and every word that contains `H` branches off from it along different 3D axes. When you want to reconstruct a sentence, you don't store the letters; you store **tiny pointers** that tell the decoder how to walk through the 3D grid and collect the bits in order.

That's the YK Nuclear Tree.

### The Three Core Components

1. **The 3D Space (Grid)** — A coordinate system `(x, y, z)` where every bit (`0` or `1`) lives at exactly one point.
2. **The Insertion Engine (Overlap)** — When new binary data arrives, the engine scans all **26 directions** in 3D space (6 straight + 12 face-diagonals + 8 corner-diagonals) to find the longest existing matching sequence. It reuses that sequence and only branches into empty space when no overlap is found.
3. **The Pointer Map (Read Instructions)** — A list of microscopic `(start_coord, direction, length)` tuples that tells the decoder exactly how to walk the grid to rebuild the original file — with zero data loss.

---

## 📐 The 26 Directions (Python prototype)

A single bit in 3D space has 26 possible neighbors — the same number of cells surrounding a cube in a 3D grid:

| Type | Count | Examples |
|------|-------|----------|
| Straight (axis-aligned) | 6 | `(+1,0,0)`, `(0,+1,0)`, `(0,0,+1)` ... |
| Face-diagonals (45° on a face) | 12 | `(+1,+1,0)`, `(+1,0,+1)`, `(0,+1,+1)` ... |
| Corner-diagonals (45° through a corner) | 8 | `(+1,+1,+1)`, `(-1,+1,+1)` ... |
| **Total** | **26** | |

---

## ⚙️ Rust Engine Internals

### Compression Pipeline

```
Raw bytes
   │
   ▼
[LZ77 with Lazy Matching]  ──► tokens (literal | match)
   │                            (length, distance) pairs
   ▼
[Canonical Huffman Coding] ──► bitstream
   │                            lit/len codes + dist codes
   ▼
[RLE of code-length trees] ──► compact header
   │                            symbols 16/17/18
   ▼
[BitWriter (LSB-first)]    ──► .yk file
```

### Why it beats ZIP on some inputs

1. **Lazy matching** — many ZIP implementations use greedy matching; the YK engine looks 1 byte ahead to find longer matches.
2. **Canonical Huffman with length limiting** — code lengths capped at 15 bits, ensuring decodability on all platforms.
3. **RLE on the header itself** — repeated code lengths (e.g., long runs of 0s for unused symbols) are RLE-encoded, shrinking the per-block header.

---

## 📊 Compression Performance

| Input Type | Expected Ratio | Notes |
|------------|----------------|-------|
| Highly repetitive text | 10–100× | Same sentence repeated many times |
| Source code | 3–10× | Repeated keywords, identifiers |
| Logs | 5–20× | Timestamps + repeated messages |
| Random / encrypted | ~1× | Information-theoretic limit (Shannon) |
| Already-compressed data | < 1× | Cannot re-compress |

---

## ⚠️ Current Limitations

- The Rust engine's Master Grid path is **hardcoded** to `/Users/yashas/Desktop/YK_Algorithm/master_dict.txt`. Edit `dict_path` in `src/main.rs` to use a different path. (CLI flag support is on the roadmap.)
- The Rust engine supports **dynamic Huffman blocks only** (BTYPE=10). Stored blocks (BTYPE=00) and fixed Huffman (BTYPE=01) are not yet implemented for decompression.
- The Python prototype uses JSON/pickle for `.yk` files, which adds metadata overhead.
- The 3D overlap scan in the Python prototype is O(n²) on grid size — needs spatial indexing for production scale.
- Random/encrypted data cannot be compressed beyond its entropy limit (Shannon's theorem).

---

## 🛣️ Roadmap

- [x] Rust decompressor (decode `.yk` back to original) ✅
- [x] Preset dictionary / Master Grid support ✅
- [ ] Make Master Grid path configurable via CLI flag (e.g. `--dict path`)
- [ ] Spatial hash index for O(1) overlap lookup in Python prototype
- [ ] GPU acceleration for 3D grid scan
- [ ] Streaming compression (compress files larger than RAM)
- [ ] CLI flags for compression level (1–9)

---

## 📜 License

MIT License — see [LICENSE](LICENSE).

## 👤 Author

**Yashas Krishnamurthy**
- Conceptualized the 3D Nuclear Tree
- Designed the Preset Dictionary architecture
- Implemented the Rust production engine and Python prototype

---

## 🙏 Acknowledgements

The 3D Nuclear Tree is an original concept — distinct from LZ77, DEFLATE, and Huffman coding, though it borrows the idea of "find the longest previous match" from LZ-family compressors. The Rust engine implements a DEFLATE-compatible bitstream for interoperability with standard inflate decoders.
