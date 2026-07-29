"""
YK Factory — the compressor side of the YK Nuclear Tree engine.
Uses block-chunking + sliding-window overlap detection for high ratios.
"""

import pickle
import os
import time


class YK_Factory:
    DIRECTIONS = [
        (1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1),
        (1, 1, 0), (1, -1, 0), (-1, 1, 0), (-1, -1, 0),
        (1, 0, 1), (1, 0, -1), (-1, 0, 1), (-1, 0, -1),
        (0, 1, 1), (0, 1, -1), (0, -1, 1), (0, -1, -1),
        (1, 1, 1), (1, 1, -1), (1, -1, 1), (1, -1, -1),
        (-1, 1, 1), (-1, 1, -1), (-1, -1, 1), (-1, -1, -1),
    ]

    MIN_OVERLAP_LEN = 8
    MAX_BRANCH_LEN = 32
    CHUNK_SIZE = 256  # bits

    def __init__(self):
        self.grid = {}
        self.keys = {}            # file_id -> list of decode instructions
        self.prefix_dict = {}     # 8-bit prefix -> list of grid coords

    def _text_to_binary(self, text: str) -> str:
        return ''.join(format(ord(c), '08b') for c in text)

    def _find_longest_overlap(self, binary_str: str):
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

    def compress(self, file_id: str, text: str):
        binary_str = self._text_to_binary(text)
        instructions = []
        i = 0
        cursor = (0, 0, 0)

        while i < len(binary_str):
            chunk = binary_str[i:i + self.MAX_BRANCH_LEN]
            overlap_coord, overlap_len, dir_idx = self._find_longest_overlap(chunk)

            if overlap_coord and overlap_len >= self.MIN_OVERLAP_LEN:
                instructions.append({
                    "start": overlap_coord,
                    "dir_idx": dir_idx,
                    "length": overlap_len,
                })
                i += overlap_len
            else:
                chosen_dir = None
                for d_idx, (dx, dy, dz) in enumerate(self.DIRECTIONS):
                    nxt = (cursor[0] + dx, cursor[1] + dy, cursor[2] + dz)
                    if nxt not in self.grid:
                        chosen_dir = d_idx
                        break
                if chosen_dir is None:
                    chosen_dir = 0

                bits_to_plant = chunk[:8]
                dx, dy, dz = self.DIRECTIONS[chosen_dir]
                curr = cursor
                for bit in bits_to_plant:
                    self.grid[curr] = bit
                    curr = (curr[0] + dx, curr[1] + dy, curr[2] + dz)
                instructions.append({
                    "start": cursor,
                    "dir_idx": chosen_dir,
                    "length": len(bits_to_plant),
                })
                i += len(bits_to_plant)
                cursor = curr

        self.keys[file_id] = instructions

    def save(self, filename: str):
        payload = {
            "creator": "Yashas Krishnamurthy",
            "format": "3D Nuclear Tree Binary Map (Factory Edition)",
            "grid": self.grid,
            "keys": self.keys,
        }
        with open(filename, "wb") as f:
            pickle.dump(payload, f)


if __name__ == "__main__":
    factory = YK_Factory()
    sample = "HELLO WORLD, I AM YASHAS. " * 100
    t0 = time.time()
    factory.compress("demo.yk", sample)
    factory.save("demo.yk")
    compressed_size = os.path.getsize("demo.yk")
    raw_size = len(sample.encode())
    print(f"Raw size      : {raw_size} bytes")
    print(f"Compressed    : {compressed_size} bytes")
    print(f"Ratio         : {raw_size / max(compressed_size, 1):.2f} : 1")
    print(f"Time          : {time.time() - t0:.2f}s")
