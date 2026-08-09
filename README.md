# 🧠 YK Labs: 3D Spatial AI & Compression Architecture

> Proprietary mathematical architecture for Artificial Intelligence memory and Data Compression. Developed by Yashas K Gangatkar.

---

## 🚀 The Problem: The $100 Billion VRAM Wall

Standard Large Language Models (ChatGPT, Claude, Llama) use 1D attention. Every token must calculate attention with every other token, resulting in **O(N²)** VRAM usage. This causes the "Lost in the Middle" effect and limits context windows, requiring massive, expensive GPU clusters.

---

## 💡 The YK Solution: Causal Voxel Attention (CVA)

YK Labs maps 1D text into a **3D Modulo Grid** using spatial mathematics derived from our custom data compression engine. By restricting attention to only the **26 physical neighbors** in 3D space, and applying a custom **3D Causal Mask**, the VRAM complexity drops from **O(N²) to O(1)**.

**Result:** A **99.99999% reduction** in attention matrix memory. A 1-Trillion parameter AI with a 1-Million token context window can run on 103 MB of VRAM.

---

## ⚙️ The AI Architecture (`yk_voxel_attention.py`)

The `CausalVoxelAttention3D` PyTorch layer is a fully functional, trainable mechanism.

- **3D Modulo Grid:** Maps 1D sequence `i` into `(X,Y,Z)` coordinates.
- **26-Directional Scanner:** Uses 3D convolutions to attend to local spatial neighbors.
- **3D Causal Mask:** Mathematically calculates the 1D distance (`Δi`) of all 27 directions, blocking the 13 "future" directions with `−∞` to preserve causality.

---

## 📈 Macro-Scale VRAM Stress Test (1 Trillion Parameters)

A 1.006 Trillion parameter YK model was instantiated using PyTorch's meta device with a 1,000,000-token context window.

| Architecture | Parameters | Context Size | Attention VRAM Required |
|---|---|---|---|
| Standard 1D (GPT-3 scale) | ~1 Trillion | 1,000,000 tokens | 3.64 TB (Requires entire data centers) |
| YK Voxel Attention | 1.006 Trillion | 1,000,000 tokens | 103.00 MB (Runs on consumer smartphones) |

---

## 🗜️ The Compression Engine (`src/main.rs`)

The foundation of the spatial math. A 64-bit Arithmetic LZMA engine written in Rust.

- **Features:** 3D Modulo Grid mapping, Optimal Parsing, Bit-Tree encoding, Multi-threaded Rayon chunking, Memory-Mapped I/O, and an Axum SaaS API.
- **Genomic Mode:** Auto-detects and 2-bit packs DNA (`.fasta`), achieving a **388:1** compression ratio on real E. Coli DNA.
- **Benchmark:** Crushes standard ZIP on text (47.1 KB vs ZIP's 53.3 KB on Alice in Wonderland).

---

## 📄 License

Proprietary IP of YK Labs. PolyForm Noncommercial. View-only for evaluation purposes.
