# YK Nuclear Tree — Architecture Deep Dive

This document explains the internal architecture of the YK Nuclear Tree
compression engine, beyond what's covered in the main README.

---

## 1. The 3D Coordinate Grid

Every bit (`0` or `1`) lives at exactly one coordinate `(x, y, z)` in an
unbounded integer grid. The grid is a Python dict:

```python
grid = {
    (0, 0, 0): "0",
    (1, 0, 0): "1",
    (2, 0, 0): "0",
    ...
}
```

### Why 3D and not 1D or 2D?

- **1D** (used by LZ77/DEFLATE) forces all data onto a single line. You can
  only walk forward or backward. Overlap detection is linear.
- **2D** gives 8 directions (4 straight + 4 diagonals). Better, but still
  limited branching capacity.
- **3D** gives 26 directions — over 3x more branching freedom than 2D and
  13x more than 1D. Shared bit sequences can be reused along many
  different geometric paths, increasing the chance of finding overlaps.

---

## 2. The 26 Direction Vectors

```python
DIRECTIONS = [
    # 6 axis-aligned (straight)
    (1, 0, 0), (-1, 0, 0),
    (0, 1, 0), (0, -1, 0),
    (0, 0, 1), (0, 0, -1),

    # 12 face-diagonals
    (1, 1, 0), (1, -1, 0), (-1, 1, 0), (-1, -1, 0),
    (1, 0, 1), (1, 0, -1), (-1, 0, 1), (-1, 0, -1),
    (0, 1, 1), (0, 1, -1), (0, -1, 1), (0, -1, -1),

    # 8 corner-diagonals
    (1, 1, 1), (1, 1, -1), (1, -1, 1), (1, -1, -1),
    (-1, 1, 1), (-1, 1, -1), (-1, -1, 1), (-1, -1, -1),
]
```

### How overlap detection uses these vectors

Given a new binary string `b` starting with bit `b[0]`:

1. Find every grid coordinate whose bit equals `b[0]`.
2. For each such coordinate, try walking in each of the 26 directions.
3. Walk forward while the bits in the grid match the bits in `b`.
4. Record the longest match found across all (coord, direction) pairs.

If the longest match ≥ `MIN_OVERLAP_LEN` (default 8 bits), record a pointer
to the existing branch. Otherwise, plant a new branch in an empty direction.

---

## 3. The Pointer Map

The pointer map is the "table of contents" that lets the decoder rebuild
the original file. Each entry is:

```json
{
  "start": [x, y, z],
  "dir_idx": 0,
  "length": 8
}
```

To decode: walk `length` bits starting from `start` in direction `dir_idx`,
concatenating the bits as you go.

### Why this is small

- A pointer is `(x, y, z) + dir_idx + length` ≈ 5 integers.
- Even with millions of pointers, the total metadata is far smaller than
  the raw binary when overlaps are frequent.

---

## 4. Compression Ratio Analysis

### Best case (highly repetitive data)

If the input is `"Hello" * 100000`:

- The first `"Hello"` (5 chars = 40 bits) gets planted in the grid.
- Every subsequent `"Hello"` is a 40-bit overlap.
- Pointer map = 100,000 tiny pointers.
- With sequence RLE, this collapses to **1 pointer + multiplier**.
- Theoretical ratio: ~10,000 : 1.

### Worst case (random/encrypted data)

Random bits have no exploitable patterns. The engine plants every bit
as a new branch, and the pointer map is as large as the original data.
Ratio ≈ 1 : 1 (no compression, slight expansion due to metadata).

### Realistic case (code, logs, text)

English text and source code have ~50% redundancy. Expect ratios of
5 : 1 to 50 : 1 depending on repetition.

---

## 5. Comparison to Existing Algorithms

| Algorithm | Dimensionality | Direction Count | Overlap Strategy |
|-----------|----------------|-----------------|------------------|
| LZ77 / DEFLATE | 1D | 2 (fwd/back) | Sliding window |
| LZW | 1D | 2 | Dictionary build |
| BWT | 1D (permuted) | 2 | Block sort |
| **YK Nuclear Tree** | **3D** | **26** | **Multi-directional grid walk** |

The YK Nuclear Tree is **not** a drop-in replacement for DEFLATE. It is an
experimental architecture that explores whether higher-dimensional overlap
detection can find patterns that 1D compressors miss.

---

## 6. Open Research Questions

1. **Is 3D optimal?** What about 4D, 5D, or n-D? Each dimension multiplies
   the direction count. 4D = 80 directions, 5D = 240 directions. More
   directions = more overlap opportunities but slower scan.

2. **Spatial indexing.** The current scan is O(grid_size × 26 ×
   match_length). A hash index on `(bit_value, next_bit_value)` would make
   overlap lookup O(1) per direction.

3. **Optimal grid packing.** Where should new branches be planted? Random
   empty cells? Near existing branches? On a regular lattice?

4. **Theoretical lower bound.** What is the theoretical best ratio for
   a 3D overlap-based compressor on i.i.d. Bernoulli sources?

---

## 7. File Format Spec (.yk v0.1)

```
.yk file (JSON v0.1)
├── creator       : string ("Yashas Krishnamurthy")
├── format        : string ("3D Nuclear Tree Binary Map")
├── grid          : { "(x, y, z)": "0" | "1", ... }
└── pointers      : { file_id: [
                        { "start": [x,y,z], "dir_idx": int, "length": int },
                        ...
                      ], ... }
```

Future binary format (v0.2, planned):
- 4-byte magic: `YKT1`
- 4-byte grid_node_count
- For each node: 3× int16 coord + 1 bit value
- 4-byte file_count
- For each file: 4-byte pointer_count + N×(3×int16 + 1 byte dir + 2 byte len)
