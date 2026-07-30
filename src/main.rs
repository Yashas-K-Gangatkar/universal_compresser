use std::fs;
use std::env;
use std::path::Path;

// ==========================================
// ENGINE 1 & 2: THE UNIVERSAL COMPILER
// ==========================================
// The Master Grid is just a raw binary file (Engine 1).
// The Compiler is the code that knows how to read/write it (Engine 2).

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("YK Universal Compiler System\nUsage:");
        println!("  Build Grid:  ./yk_engine build <input_file>");
        println!("  Get Ticket:  ./yk_engine ticket <input_file> <output.ticket>");
        println!("  Scan/Decode: ./yk_engine scan <input.ticket> <output_file>");
        return;
    }

    let command = &args[1];
    let grid_path = "yk_master.grid";

    match command.as_str() {
        "build" => {
            let input_file = &args[2];
            let data = fs::read(input_file).expect("Failed to read input file");

            // Read existing grid (or start empty)
            let mut grid_data = fs::read(grid_path).unwrap_or_else(|_| Vec::new());
            let start_offset = grid_data.len();

            // Append raw bytes to the Master Grid (NO POINTERS ADDED TO GRID)
            grid_data.extend_from_slice(&data);

            // Save the updated Master Grid
            fs::write(grid_path, &grid_data).expect("Failed to write Master Grid");

            // Save the metadata (Start offset + Length) to the Compiler's index
            let index_entry = format!("{}|{}\n", start_offset, data.len());
            let mut index_data = fs::read_to_string("yk_compiler.index").unwrap_or_default();
            index_data.push_str(&index_entry);
            fs::write("yk_compiler.index", index_data).expect("Failed to write index");

            println!("Engine 1 (Mapper): Added {} bytes to Master Grid at offset {}.", data.len(), start_offset);
            println!("Engine 2 (Compiler): Memory updated.");
        }

        "ticket" => {
            if args.len() < 4 {
                println!("Usage: ./yk_engine ticket <input_file> <output.ticket>");
                return;
            }
            let input_file = &args[2];
            let output_ticket = &args[3];
            let data = fs::read(input_file).expect("Failed to read input file");

            let grid_data = fs::read(grid_path).expect("Master Grid not found. Run 'build' first.");
            let index_data = fs::read_to_string("yk_compiler.index").expect("Compiler index not found.");

            // Search the Master Grid for the exact file
            let mut found_offset: Option<usize> = None;
            for line in index_data.lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 2 {
                    let offset: usize = parts[0].parse().unwrap();
                    let length: usize = parts[1].parse().unwrap();

                    if length == data.len() && grid_data[offset..offset+length] == data[..] {
                        found_offset = Some(offset);
                        break;
                    }
                }
            }

            if let Some(offset) = found_offset {
                // ENGINE 2 generates a 9-byte ticket!
                // Format: [4 bytes offset] [4 bytes length] [1 byte flag]
                let mut ticket_bytes = Vec::new();
                ticket_bytes.extend_from_slice(&(offset as u32).to_le_bytes());
                ticket_bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
                ticket_bytes.push(1); // Flag 1 = Match found in Master Grid

                fs::write(output_ticket, &ticket_bytes).expect("Failed to write ticket");
                println!("Engine 2 (Compiler): File recognized! Generated 9-byte ticket.");
                println!("Original Size: {} bytes", data.len());
                println!("Ticket Size: 9 bytes");
                println!("Ratio: {:.1} : 1", data.len() as f64 / 9.0);
            } else {
                println!("File not found in Master Grid. Run 'build' first.");
            }
        }

        "scan" => {
            if args.len() < 4 {
                println!("Usage: ./yk_engine scan <input.ticket> <output_file>");
                return;
            }
            let input_ticket = &args[2];
            let output_file = &args[3];

            let ticket_data = fs::read(input_ticket).expect("Failed to read ticket");
            if ticket_data.len() != 9 || ticket_data[8] != 1 {
                println!("Invalid ticket.");
                return;
            }

            let offset = u32::from_le_bytes([ticket_data[0], ticket_data[1], ticket_data[2], ticket_data[3]]) as usize;
            let length = u32::from_le_bytes([ticket_data[4], ticket_data[5], ticket_data[6], ticket_data[7]]) as usize;

            let grid_data = fs::read(grid_path).expect("Master Grid not found.");

            // Engine 2 extracts the exact bytes from the Master Grid
            let extracted_data = &grid_data[offset..offset+length];

            fs::write(output_file, extracted_data).expect("Failed to write output file");
            println!("Engine 2 (Scanner): Read 9-byte ticket, extracted {} bytes from Master Grid.", length);
            println!("Zero Data Loss: Achieved.");
        }

        _ => println!("Unknown command. Use 'build', 'ticket', or 'scan'."),
    }
}
