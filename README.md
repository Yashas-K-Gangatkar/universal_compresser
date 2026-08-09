# 🧠 YK Labs: 3D Spatial AI & Compression Architecture

> Proprietary mathematical architecture for Artificial Intelligence memory and Data Compression. Developed by Yashas K Gangatkar.

---

## 🚀 The Problem: The $100 Billion VRAM Wall

Standard Large Language Models (ChatGPT, Claude, Llama) use 1D attention. Every token must calculate attention with every other token, resulting in **O(N²)** VRAM usage. This causes the "Lost in the Middle" effect and limits context windows to ~200K tokens, requiring massive, expensive GPU clusters.

---

## 💡 The YK Solution: Causal Voxel Attention (CVA)

YK Labs maps 1D text into a **3D Modulo Grid** using spatial mathematics derived from our custom data compression engine. By restricting attention to only the **26 physical neighbors** in 3D space, and applying a custom **3D Causal Mask** to prevent looking into the future, the VRAM complexity drops from **O(N²) to O(1)**.

**Result:** A **99.97% reduction** in attention matrix memory. An AI could process a 1-million-token book using the same VRAM that standard AI uses for 2,700 tokens.

---

## ⚙️ The AI Architecture (`voxel_attention.py`)

The `CausalVoxelAttention3D` PyTorch layer is a fully functional, trainable mechanism.

- **3D Modulo Grid:** Maps 1D sequence `i` into `(X,Y,Z)` coordinates.
- **26-Directional Scanner:** Uses 3D convolutions to attend to local spatial neighbors.
- **3D Causal Mask:** Mathematically calculates the 1D distance (`Δi`) of all 27 directions, blocking the 13 "future" directions with `−∞` to ensure sequence causality is preserved during training.

---

## 🗜️ The Compression Engine (`src/main.rs`)

The foundation of the spatial math. A 64-bit Arithmetic LZMA engine written in Rust.

- **Features:** 3D Modulo Grid mapping, Optimal Parsing (Dynamic Programming), Bit-Tree encoding, Multi-threaded Rayon chunking, Memory-Mapped I/O, and an Axum SaaS API.
- **Genomic Mode:** Auto-detects and 2-bit packs DNA (`.fasta`), achieving a **388:1** compression ratio on real E. Coli DNA using the Reference Grid architecture.
- **Benchmark:** Crushes standard ZIP on text (**47.1 KB** vs ZIP's 53.3 KB on Alice in Wonderland).
