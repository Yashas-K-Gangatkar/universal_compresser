"""
YK Nuclear Tree Algorithm
=========================
A 3D multi-dimensional binary mapping algorithm for data compression.

Conceptualized by: Yashas Krishnamurthy
Format: 3D Nuclear Tree Binary Map (.yk)

The algorithm maps binary data into a 3D coordinate space (X, Y, Z) where
bits (0s and 1s) are stored as nodes. When new data is inserted, the engine
scans all 26 possible directions (6 straight + 12 face-diagonals + 8 corner-
diagonals) to find overlapping bit sequences, branching only when no overlap
is found. This creates a "nuclear tree" structure where shared data lives
once and is referenced via microscopic pointer maps.
"""

import json
import os
import random


class YK_Nuclear_Tree:
    """Core 3D Nuclear Tree compression engine.

    Maps binary data into a 3D coordinate grid where overlapping bit
    sequences share coordinates, dramatically reducing storage requirements.
    """

    # 26 directions in 3D space:
    #   - 6 straight (axis-aligned)
    #   - 12 face-diagonals (45 degrees on a face)
    #   - 8 corner-diagonals (45 degrees through a corner)
    DIRECTIONS = [
        # 6 straight
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

    MIN_OVERLAP_LEN = 8   # bits — minimum overlap worth reusing
    MAX_BRANCH_LEN = 32   # bits — max bits planted in a single new branch

    def __init__(self):
        # 3D grid: (x, y, z) -> '0' | '1'
        self.grid = {}
        # Pointer map: file_id -> list of decode instructions
        self.pointers = {}

    # ---------- text <-> binary helpers ----------
    def _text_to_binary(self, text: str) -> str:
        return ''.join(format(ord(c), '08b') for c in text)

    def _binary_to_text(self, binary_str: str) -> str:
        chars = [binary_str[i:i + 8] for i in range(0, len(binary_str), 8)]
        return ''.join(chr(int(c, 2)) for c in chars if len(c) == 8)

    # ---------- core: find longest overlap in 3D grid ----------
    def _find_longest_overlap(self, binary_str: str):
        """Scan all 26 directions from every grid node to find the longest
        prefix of `binary_str` that already exists in the 3D grid."""
        best_match = None
        max_length = 0
        best_dir_idx = -1

        for coord, bit in self.grid.items():
            if bit != binary_str[0]:
                continue
            for d_idx, (dx, dy, dz) in enumerate(self.DIRECTIONS):
                length = 1
                curr = coord
                while length < len(binary_str):
                    nxt = (curr[0] + dx, curr[1] + dy, curr[2] + dz)
                    if nxt in self.grid and self.grid[nxt] == binary_str[length]:
                        length += 1
                        curr = nxt
                    else:
                        break
                if length > max_length:
                    max_length = length
                    best_match = coord
                    best_dir_idx = d_idx
        return best_match, max_length, best_dir_idx

    # ---------- core: insert binary into 3D grid ----------
    def _plant_branch(self, binary_str: str, start_coord, dir_idx: int):
        """Plant a new branch of bits starting at `start_coord`, walking
        in direction `dir_idx`. Used when no overlap was found."""
        dx, dy, dz = self.DIRECTIONS[dir_idx]
        curr = start_coord
        path = [(curr, "plant")]
        for bit in binary_str:
            self.grid[curr] = bit
            curr = (curr[0] + dx, curr[1] + dy, curr[2] + dz)
        return path

    # ---------- public: insert a file ----------
    def insert(self, file_id: str, text: str):
        """Convert text to binary and walk it into the 3D grid, reusing
        existing overlaps whenever possible."""
        binary_str = self._text_to_binary(text)
        instructions = []
        i = 0
        cursor = (0, 0, 0)  # arbitrary starting anchor

        while i < len(binary_str):
            chunk = binary_str[i:i + self.MAX_BRANCH_LEN]
            overlap_coord, overlap_len, dir_idx = self._find_longest_overlap(chunk)

            if overlap_coord and overlap_len >= self.MIN_OVERLAP_LEN:
                # Reuse existing branch
                instructions.append({
                    "type": "overlap",
                    "start": overlap_coord,
                    "dir_idx": dir_idx,
                    "length": overlap_len,
                })
                i += overlap_len
            else:
                # Plant new branch in an unused direction
                # Find a direction where the next cell is empty
                chosen_dir = None
                for d_idx, (dx, dy, dz) in enumerate(self.DIRECTIONS):
                    nxt = (cursor[0] + dx, cursor[1] + dy, cursor[2] + dz)
                    if nxt not in self.grid:
                        chosen_dir = d_idx
                        break
                if chosen_dir is None:
                    chosen_dir = 0  # fallback

                # Plant one byte (8 bits) at a time
                bits_to_plant = chunk[:8]
                self._plant_branch(bits_to_plant, cursor, chosen_dir)
                instructions.append({
                    "type": "plant",
                    "start": cursor,
                    "dir_idx": chosen_dir,
                    "length": len(bits_to_plant),
                })
                i += len(bits_to_plant)
                # advance cursor in the chosen direction
                dx, dy, dz = self.DIRECTIONS[chosen_dir]
                cursor = (cursor[0] + dx * len(bits_to_plant),
                          cursor[1] + dy * len(bits_to_plant),
                          cursor[2] + dz * len(bits_to_plant))

        self.pointers[file_id] = instructions

    # ---------- public: decode a file ----------
    def decode(self, file_id: str) -> str:
        """Walk the 3D grid using the pointer map to reconstruct text."""
        if file_id not in self.pointers:
            return ""
        binary_str = ""
        for step in self.pointers[file_id]:
            cx, cy, cz = step["start"]
            dx, dy, dz = self.DIRECTIONS[step["dir_idx"]]
            for _ in range(step["length"]):
                binary_str += self.grid[(cx, cy, cz)]
                cx += dx
                cy += dy
                cz += dz
        return self._binary_to_text(binary_str)

    # ---------- public: save / load .yk files ----------
    def save_yk(self, filename: str):
        """Save the grid + pointer map as a .yk JSON file."""
        payload = {
            "creator": "Yashas Krishnamurthy",
            "format": "3D Nuclear Tree Binary Map",
            "grid": {str(k): v for k, v in self.grid.items()},
            "pointers": self.pointers,
        }
        with open(filename, "w") as f:
            json.dump(payload, f, indent=2)

    @classmethod
    def load_yk(cls, filename: str) -> "YK_Nuclear_Tree":
        """Load a .yk file and rebuild the engine state."""
        with open(filename) as f:
            payload = json.load(f)
        tree = cls()
        for k, v in payload["grid"].items():
            # parse "(x, y, z)" back to tuple
            x, y, z = map(int, k.strip("()").split(","))
            tree.grid[(x, y, z)] = v
        tree.pointers = payload["pointers"]
        return tree

    # ---------- stats ----------
    def stats(self):
        return {
            "grid_size": len(self.grid),
            "files_stored": len(self.pointers),
            "directions_used": len(self.DIRECTIONS),
        }


# ---------- Demo ----------
if __name__ == "__main__":
    tree = YK_Nuclear_Tree()

    sample_text = "HELLO WORLD, I AM YASHAS."
    print(f"Original text: {sample_text}")
    print(f"Original size: {len(sample_text)} bytes")

    tree.insert("file1.yk", sample_text)
    decoded = tree.decode("file1.yk")
    print(f"Decoded text : {decoded}")
    print(f"Lossless     : {decoded == sample_text}")
    print(f"Stats        : {tree.stats()}")

    # Save & reload test
    tree.save_yk("example.yk")
    print("\nSaved to example.yk")
    reloaded = YK_Nuclear_Tree.load_yk("example.yk")
    print(f"Reloaded decode: {reloaded.decode('file1.yk')}")
    print(f"Lossless after reload: {reloaded.decode('file1.yk') == sample_text}")
