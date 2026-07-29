# 🌳 YK Nuclear Tree — 3D Binary Compression Engine

> **A 3D multi-dimensional binary mapping algorithm** that compresses data by storing shared bit sequences in a 3D coordinate space (X, Y, Z) and reconstructing files via microscopic pointer maps.

**Creator:** Yashas Krishnamurthy
**Format:** `.yk` (3D Nuclear Tree Binary Map)
**Status:** Concept / Prototype
**License:** MIT

---

## 🧠 The Concept

Imagine standing inside a 3D grid. Words like `HELP`, `HOPE`, and `HUGE` appear as physical branches growing from a shared letter `H`. The `H` is **stationary** — planted once at a single coordinate — and every word that contains `H` branches off from it along different 3D axes. When you want to reconstruct a sentence, you don't store the letters; you store **tiny pointers** that tell the decoder how to walk through the 3D grid and collect the bits in order.

That's the YK Nuclear Tree.

### The Three Core Components

1. **The 3D Space (Grid)** — A coordinate system `(x, y, z)` where every bit (`0` or `1`) lives at exactly one point.
2. **The Insertion Engine (Overlap)** — When new binary data arrives, the engine scans all **26 directions** in 3D space (6 straight + 12 face-diagonals + 8 corner-diagonals) to find the longest existing matching sequence. It reuses that sequence and only branches into empty space when no overlap is found.
3. **The Pointer Map (Read Instructions)** — A list of microscopic `(start_coord, direction, length)` tuples that tells the decoder exactly how to walk the grid to rebuild the original file — with zero data loss.

---

## 📐 The 26 Directions

A single bit in 3D space has 26 possible neighbors — the same number of cells surrounding a cube in a 3D grid:

| Type | Count | Examples |
|------|-------|----------|
| Straight (axis-aligned) | 6 | `(+1,0,0)`, `(0,+1,0)`, `(0,0,+1)` ... |
| Face-diagonals (45° on a face) | 12 | `(+1,+1,0)`, `(+1,0,+1)`, `(0,+1,+1)` ... |
| Corner-diagonals (45° through a corner) | 8 | `(+1,+1,+1)`, `(-1,+1,+1)` ... |
| **Total** | **26** | |

When inserting data, the engine tries all 26 directions from every existing matching bit to find the longest overlap.

---

## 🚀 Quick Start

```bash
# Clone
git clone https://github.com/YASHAS-KR/yk-nuclear-tree.git
cd yk-nuclear-tree/src

# Run the basic demo (creates example.yk)
python yk_nuclear_tree.py

# Run the 10MB compression test
python yk_10mb_test.py
```

### Expected output from `yk_nuclear_tree.py`:
```
Original text: HELLO WORLD, I AM YASHAS.
Original size: 25 bytes
Decoded text : HELLO WORLD, I AM YASHAS.
Lossless     : True
Stats        : {'grid_size': 200, 'files_stored': 1, 'directions_used': 26}
```

---

## 📁 Project Structure

```
yk-nuclear-tree/
├── README.md
├── LICENSE
├── .gitignore
├── src/
│   ├── yk_nuclear_tree.py     # Core engine (single-file demo)
│   ├── yk_factory.py          # Compressor (block-chunking edition)
│   ├── yk_machine.py          # Decoder
│   └── yk_10mb_test.py        # 10MB compression benchmark
├── examples/
│   └── example.yk             # Sample compressed file (after running demo)
└── docs/
    └── ARCHITECTURE.md        # Algorithm deep-dive
```

---

## 🔬 How It Works (Step by Step)

### Step 1 — Text → Binary
```
"HI"  →  01001000 01001001  (16 bits)
```

### Step 2 — Find Overlap in 3D Grid
The engine scans every existing bit in the grid. For each bit that matches the first bit of the new data, it walks all 26 directions to find the longest continuous match.

### Step 3 — Plant or Reuse
- **If overlap ≥ 8 bits** → record a pointer to the existing branch.
- **Else** → plant a new branch in an empty direction.

### Step 4 — Save the Pointer Map
The `.yk` file contains:
```json
{
  "creator": "Yashas Krishnamurthy",
  "format": "3D Nuclear Tree Binary Map",
  "grid": {"(0, 0, 0)": "0", "(1, 0, 0)": "1", ...},
  "pointers": {
    "file1.yk": [
      {"start": [0,0,0], "dir_idx": 0, "length": 8},
      {"start": [3,-3,0], "dir_idx": 9, "length": 8}
    ]
  }
}
```

### Step 5 — Decode (Zero Loss)
The decoder walks the grid using the pointer map and reconstructs the exact original binary string — bit for bit.

---

## 🎯 Compression Strategy

The engine combines multiple techniques for high ratios:

1. **3D Overlap Detection** — Find the longest existing matching sequence in any of 26 directions.
2. **Block Chunking** — Scan in 8-bit (1-byte) chunks to avoid alignment bugs.
3. **Sliding Window** — Look back through the grid for any previous occurrence of the current sequence.
4. **Sequence RLE** — When the same sequence repeats back-to-back, store one pointer + a multiplier.
5. **Preset Dictionary (Roadmap)** — Reference an external "Master Grid" of common patterns; files that match shrink to a microscopic 9-byte ticket.

### Known Compression Behaviors

| Input Type | Expected Ratio | Notes |
|------------|----------------|-------|
| Highly repetitive text | Up to 10,000 : 1 | Same sentence repeated 100K times |
| Code / Logs | 5 – 50 : 1 | Lots of repeated keywords |
| Random binary (encrypted) | ~1 : 1 | Information-theoretic limit |
| Already-compressed data | < 1 : 1 | Cannot re-compress |

---

## ⚠️ Current Limitations

- The current implementation uses JSON/pickle for `.yk` files, which adds metadata overhead.
- The 26-direction scan is O(n²) on grid size — needs spatial indexing (e.g., hash on bit value) for production scale.
- Random/encrypted data cannot be compressed beyond its entropy limit (Shannon's theorem).

---

## 🛣️ Roadmap

- [ ] Binary serialization (replace JSON with packed bytes)
- [ ] Spatial hash index for O(1) overlap lookup
- [ ] Preset dictionary / Master Grid support
- [ ] Rust port for production speed
- [ ] GPU acceleration for 3D grid scan

---

## 📜 License

MIT License — see [LICENSE](LICENSE).

## 👤 Author

**Yashas Krishnamurthy**
- Conceptualized the 3D Nuclear Tree
- Designed the Preset Dictionary architecture
- Implemented the Python prototype

---

## 🙏 Acknowledgements

This algorithm was conceptualized through a series of design conversations exploring multi-dimensional binary mapping. The 3D Nuclear Tree is an original concept — distinct from LZ77, DEFLATE, and Huffman coding, though it borrows the idea of "find the longest previous match" from LZ-family compressors.
