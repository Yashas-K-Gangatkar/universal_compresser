"""
YK 10MB Compression Test
========================
Generates a 10MB repetitive text file and compresses it with the YK Factory,
then decodes it with the YK Machine to verify zero data loss.

Run:
    python yk_10mb_test.py
"""

import os
import time

from yk_factory import YK_Factory
from yk_machine import YK_Machine


def main():
    # --- 1. Generate a 10 MB text file ---
    print("Generating 10MB of repetitive text data...")
    sample_text = (
        "This is the YK Nuclear Tree algorithm compressing data in 3D space. "
        "Yashas Krishnamurthy is the inventor of this format. "
    )
    huge_text = sample_text * 100_000

    raw_file = "10mb_raw.txt"
    with open(raw_file, "w") as f:
        f.write(huge_text)
    raw_size = os.path.getsize(raw_file)
    print(f"Raw file created: {raw_size} bytes ({raw_size / 1024 / 1024:.2f} MB)")

    # --- 2. Compress ---
    print("\nRunning YK Factory...")
    t0 = time.time()
    factory = YK_Factory()
    factory.compress("10mb_data.yk", huge_text)
    factory.save("10mb_data.yk")
    compressed_size = os.path.getsize("10mb_data.yk")
    elapsed = time.time() - t0
    print(f"Manufactured 10mb_data.yk - Size: {compressed_size} bytes")
    print(f"Compression took {elapsed:.2f} seconds.")

    # --- 3. Decode & verify ---
    print("\nRunning YK Machine to verify zero-loss decode...")
    machine = YK_Machine()
    machine.load_yk("10mb_data.yk")
    decoded = machine.decode("10mb_data.yk")
    lossless = decoded == huge_text

    # --- 4. Report ---
    ratio = raw_size / max(compressed_size, 1)
    print("\n--- FINAL RESULTS ---")
    print(f"Original Size : {raw_size} bytes ({raw_size / 1024 / 1024:.2f} MB)")
    print(f"Compressed Size: {compressed_size} bytes ({compressed_size / 1024:.2f} KB)")
    print(f"Compression Ratio: {ratio:.2f} : 1")
    print(f"Lossless decode  : {lossless}")
    print(f"Target ratio     : {raw_size / 1024:.0f} : 1 (10MB -> 1KB)")


if __name__ == "__main__":
    main()
