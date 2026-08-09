import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
import time
from yk_voxel_attention import YKLanguageModel

if __name__ == "__main__":
    torch.set_num_threads(10)
    device = torch.device("mps" if torch.backends.mps.is_available() else "cpu")
    print(f"Device: {device}")
    print(f"PyTorch Active Threads (Cores): {torch.get_num_threads()}")

    GRID_X, GRID_Y, GRID_Z = 10, 10, 10
    SEQ_LEN = GRID_X * GRID_Y * GRID_Z
    EMBED_DIM = 1024
    VOCAB_SIZE = 50000
    NUM_LAYERS = 8

    print(f"Initializing YK 433M Model...")
    model = YKLanguageModel(VOCAB_SIZE, EMBED_DIM, GRID_X, GRID_Y, GRID_Z, num_layers=NUM_LAYERS).to(device)

    total_params = sum(p.numel() for p in model.parameters())
    print(f"Total Parameters: {total_params:,} ({total_params / 1e6:.2f} Million)")

    print(f"Generating {SEQ_LEN} token batch...")
    x_batch = torch.randint(0, VOCAB_SIZE, (1, SEQ_LEN)).to(device)
    y_batch = torch.randint(0, VOCAB_SIZE, (1, SEQ_LEN)).to(device)

    optimizer = optim.AdamW(model.parameters(), lr=1e-4)
    criterion = nn.CrossEntropyLoss()

    print("Starting Multi-Core Forward/Backward pass...")
    model.train()
    start_time = time.time()

    logits = model(x_batch)
    loss = criterion(logits.view(-1, VOCAB_SIZE), y_batch.view(-1))

    optimizer.zero_grad()
    loss.backward()
    optimizer.step()

    end_time = time.time()

    print(f"\n>>> SUCCESS: {total_params / 1e6:.2f} Million parameter training step completed! <<<")
    print(f"Loss: {loss.item():.4f}")
    print(f"Time taken using 10 cores: {end_time - start_time:.2f} seconds")
