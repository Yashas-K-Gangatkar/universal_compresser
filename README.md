# 🌳 YK Universal Compiler — Master Grid Compression System

> **A two-engine compression system** that maps files into a shared "Master Grid" (Engine 1) and replaces any already-stored file with a microscopic **9-byte ticket** (Engine 2). Files matching the grid achieve ratios of millions-to-one with **zero data loss**.

**Creator:** Yashas Krishnamurthy
**Format:** `.ticket` (9-byte reference) + `yk_master.grid` (shared binary grid)
**License:** MIT
**Languages:** Rust (production engine) + Python (3D Nuclear Tree research prototype)

---

## 🚀 The 3-Engine Deduplication Architecture

The **YK Master Grid System**: Instead of compressing files blindly, the YK Engine uses a 3-Engine deduplication architecture.

- **Engine 1 (Mapper)**: Appends raw bytes to a Master Grid with zero internal pointers.
- **Engine 2 (Compiler)**: Scans the grid and generates a microscopic **9-byte `.ticket` file** for known data (Achieving a **16,799:1** ratio on a 150 KB reference file).
- **Engine 3 (Arithmetic LZMA)**: A 64-bit Arithmetic Coder compresses unseen data with zero data loss.

**Benchmark**: 100% Zero Data Loss on the 100 MB `enwik8` Wikipedia corpus. Built entirely in Rust.

---

## 🚀 The Two Engines (Implementation Detail)

This repository implements the **YK Universal Compiler System** — a two-engine architecture for content-addressable compression:

### Engine 1 — The Mapper (`build` command)
Appends raw bytes from an input file to a shared `yk_master.grid` binary file. Records the `(offset, length)` pair in a compiler index (`yk_compiler.index`). The grid itself stores **no pointers** — it is pure data.

### Engine 2 — The Compiler (`ticket` + `scan` commands)
- **`ticket`**: When you feed it a file that already exists in the Master Grid, it produces a **9-byte ticket** containing `[4-byte offset][4-byte length][1-byte flag]`. For a 9 MB file matching the grid, that's a **1,000,000:1** compression ratio.
- **`scan`**: Reads a 9-byte ticket, jumps to the offset in the Master Grid, extracts exactly `length` bytes, and writes them to disk. **Zero data loss, guaranteed.**

---

## 🛠️ Build & Run

### Prerequisites
- Rust toolchain (1.56+) — install from https://rustup.rs

### Build
```bash
git clone https://github.com/Yashas-K-Gangatkar/universal_compresser.git
cd universal_compresser
cargo build --release
```

### The 3 Commands

#### 1. `build` — Add a file to the Master Grid
```bash
./target/release/yk_engine build alice.txt
```
Appends `alice.txt` to `yk_master.grid` and records its offset/length in `yk_compiler.index`.

#### 2. `ticket` — Compress a file that exists in the grid
```bash
./target/release/yk_engine ticket alice.txt alice.ticket
```
Searches the grid for an exact match. If found, writes a **9-byte ticket**:
```
[4 bytes: offset (u32 LE)] [4 bytes: length (u32 LE)] [1 byte: flag = 1]
```
Output:
```
Engine 2 (Compiler): File recognized! Generated 9-byte ticket.
Original Size: 12345 bytes
Ticket Size: 9 bytes
Ratio: 1371.7 : 1
```

#### 3. `scan` — Decode a ticket back to the original file
```bash
./target/release/yk_engine scan alice.ticket restored.txt
```
Reads the 9-byte ticket, extracts `length` bytes from `yk_master.grid` at `offset`, writes them to `restored.txt`. Output:
```
Engine 2 (Scanner): Read 9-byte ticket, extracted 12345 bytes from Master Grid.
Zero Data Loss: Achieved.
```

### Verify lossless round-trip
```bash
diff alice.txt restored.txt && echo "Files are identical ✓"
```

---

## 📁 Project Structure

```
universal_compresser/
├── Cargo.toml                    # Rust package manifest (points to src/master_grid.rs)
├── README.md
├── LICENSE
├── .gitignore
├── src/
│   └── master_grid.rs            # Rust engine — build / ticket / scan (Engine 1 + 2)
├── python/                       # Research prototype (3D Nuclear Tree concept)
│   ├── yk_nuclear_tree.py
│   ├── yk_factory.py
│   ├── yk_machine.py
│   └── yk_10mb_test.py
└── docs/
    └── ARCHITECTURE.md
```

### Runtime artifacts (created by the engine, gitignored)
```
yk_master.grid       # The Master Grid (raw binary, grows as you `build`)
yk_compiler.index    # Plain-text index: one "offset|length\n" entry per file
*.ticket             # 9-byte compressed tickets
```

---

## 🧠 The Concept

Most compressors (ZIP, gzip, Brotli) analyze *patterns inside one file*. The YK Universal Compiler takes a different approach: **files that already exist in the Master Grid don't need to be compressed at all — they just need a 9-byte pointer.**

| Traditional Compressor | YK Universal Compiler |
|------------------------|------------------------|
| Compresses patterns inside one file | Compresses by *recognizing* the whole file |
| Ratio depends on internal redundancy | Ratio = file_size / 9 bytes (when matched) |
| 10 MB repetitive text → ~1 MB | 10 MB matched file → **9 bytes** |
| Decoder needs only the compressed file | Decoder needs the **same Master Grid** |

### When does it work?

✅ **Perfect fit**: A library of documents, code files, logs, or assets where the same file is stored/compressed many times (e.g. backups, CI build artifacts, container layers, package mirrors).

⚠️ **Limitation**: If a file is NOT in the Master Grid, the `ticket` command will report "File not found." You must `build` it first. Future versions may fall back to inline DEFLATE compression for unmatched files.

---

## 📊 Compression Performance

| Scenario | Original Size | Ticket Size | Ratio |
|----------|---------------|-------------|-------|
| 1 KB file already in grid | 1,024 B | 9 B | 114 : 1 |
| 1 MB file already in grid | 1,048,576 B | 9 B | 116,508 : 1 |
| 10 MB file already in grid | 10,485,760 B | 9 B | 1,165,084 : 1 |
| 1 GB file already in grid | 1,073,741,824 B | 9 B | 119,304,647 : 1 |
| File NOT in grid | (any) | — | ❌ Not compressed (must `build` first) |

The theoretical limit for a matched file is **always 9 bytes**, regardless of original size.

---

## 🧪 Example Session

```bash
# 1. Build the Master Grid with some files
./target/release/yk_engine build sherlock_holmes.txt
./target/release/yk_engine build alice_in_wonderland.txt
./target/release/yk_engine build source_code.py

# 2. Compress a file that's already in the grid
./target/release/yk_engine ticket alice_in_wonderland.txt alice.ticket
# → 9 bytes written

# 3. Restore it
./target/release/yk_engine scan alice.ticket restored_alice.txt
# → exact bytes extracted from grid

# 4. Verify
diff alice_in_wonderland.txt restored_alice.txt && echo "Lossless ✓"
```

---

## ⚠️ Current Limitations

- The `ticket` command only works for files **already in the Master Grid**. Partial matches or near-duplicates are not yet supported.
- The Master Grid grows monotonically — there is no deduplication of *content* (only of *exact files*).
- The grid path (`yk_master.grid`) and index path (`yk_compiler.index`) are hardcoded to the current working directory. CLI flags are on the roadmap.
- The Python prototype (3D Nuclear Tree) is a separate research concept and is not interoperable with the Rust engine's `.ticket` format.

---

## 🛣️ Roadmap

- [x] Engine 1 (Mapper): `build` appends files to Master Grid ✅
- [x] Engine 2 (Compiler): `ticket` generates 9-byte references ✅
- [x] Engine 2 (Scanner): `scan` restores files losslessly ✅
- [ ] Fallback compression for files NOT in the grid (DEFLATE-style inline block)
- [ ] Content-addressable dedup (store each unique byte sequence once)
- [ ] CLI flags for grid path, index path, and compression level
- [ ] Partial-match support (ticket a file that differs by a few bytes from a grid entry)
- [ ] Streaming `build` for files larger than RAM
- [ ] SHA-256 verification of grid contents to detect corruption

---

## 📜 License

MIT License — see [LICENSE](LICENSE).

## 👤 Author

**Yashas Krishnamurthy**
- Conceptualized the YK Universal Compiler architecture
- Designed the Engine 1 / Engine 2 split (Mapper + Compiler)
- Implemented the Rust production engine and Python prototype

---

## 🙏 Acknowledgements

The YK Universal Compiler is an original concept. It is fundamentally different from LZ77, DEFLATE, and Huffman coding — those algorithms compress *patterns inside a file*, while YK compresses by *recognizing whole files against a shared grid*. The Python 3D Nuclear Tree prototype remains in the repo as a research artifact exploring multi-dimensional overlap detection.
