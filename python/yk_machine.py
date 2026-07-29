"""
YK Machine — the decoder side of the YK Nuclear Tree engine.
Loads a .yk file and reconstructs the original text using the pointer map.
"""

import pickle


class YK_Machine:
    DIRECTIONS = [
        (1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1),
        (1, 1, 0), (1, -1, 0), (-1, 1, 0), (-1, -1, 0),
        (1, 0, 1), (1, 0, -1), (-1, 0, 1), (-1, 0, -1),
        (0, 1, 1), (0, 1, -1), (0, -1, 1), (0, -1, -1),
        (1, 1, 1), (1, 1, -1), (1, -1, 1), (1, -1, -1),
        (-1, 1, 1), (-1, 1, -1), (-1, -1, 1), (-1, -1, -1),
    ]

    def __init__(self):
        self.grid = {}
        self.keys = {}

    def load_yk(self, filename: str):
        with open(filename, "rb") as f:
            yk_data = pickle.load(f)
        self.grid = yk_data["grid"]
        self.keys = yk_data["keys"]
        print(f"Loaded {filename} into the YK Machine.")

    def _binary_to_text(self, binary_str: str) -> str:
        chars = [binary_str[i:i + 8] for i in range(0, len(binary_str), 8)]
        return ''.join(chr(int(c, 2)) for c in chars if len(c) == 8)

    def decode(self, file_id: str) -> str:
        if file_id not in self.keys:
            return "File not found"
        instructions = self.keys[file_id]
        binary_str = ""
        for step in instructions:
            cx, cy, cz = step["start"]
            dx, dy, dz = self.DIRECTIONS[step["dir_idx"]]
            for _ in range(step["length"]):
                binary_str += self.grid[(cx, cy, cz)]
                cx += dx
                cy += dy
                cz += dz
        return self._binary_to_text(binary_str)


if __name__ == "__main__":
    machine = YK_Machine()
    machine.load_yk("demo.yk")
    decoded = machine.decode("demo.yk")
    print(f"Decoded length: {len(decoded)} chars")
    print(f"First 60 chars: {decoded[:60]}")
