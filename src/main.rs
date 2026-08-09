use std::fs;
use std::env;
use std::collections::HashMap;

// ==========================================
// 1D TO 3D MAPPING (Modulo Grid)
// ==========================================
fn offset_to_3d(off: usize) -> (u32, u32, u32) {
    let x = (off % 1000000) as u32;
    let y = ((off / 1000000) % 1000000) as u32;
    let z = (off / 1000000000000) as u32;
    (x, y, z)
}

fn to_3d_offset(x: u32, y: u32, z: u32) -> usize {
    (x as usize) + (y as usize * 1000000) + (z as usize * 1000000000000)
}

// ==========================================
// THE 26 DIRECTIONS
// ==========================================
const DIRECTIONS: [(i32, i32, i32); 26] = [
    (1,0,0), (-1,0,0), (0,1,0), (0,-1,0), (0,0,1), (0,0,-1),
    (1,1,0), (1,-1,0), (-1,1,0), (-1,-1,0),
    (1,0,1), (1,0,-1), (-1,0,1), (-1,0,-1),
    (0,1,1), (0,1,-1), (0,-1,1), (0,-1,-1),
    (1,1,1), (1,1,-1), (1,-1,1), (1,-1,-1),
    (-1,1,1), (-1,1,-1), (-1,-1,1), (-1,-1,-1)
];

// ==========================================
// 64-BIT ARITHMETIC CODER
// ==========================================
const PRECISION: u32 = 32;
const WHOLE: u64 = 1u64 << PRECISION;
const HALF: u64 = 1u64 << (PRECISION - 1);
const QUARTER: u64 = 1u64 << (PRECISION - 2);

struct BitWriter { buffer: Vec<u8>, current_byte: u8, bit_count: u8 }
impl BitWriter {
    fn new() -> Self { BitWriter { buffer: Vec::new(), current_byte: 0, bit_count: 0 } }
    fn write_bit(&mut self, bit: u8) {
        self.current_byte |= (bit & 1) << self.bit_count;
        self.bit_count += 1;
        if self.bit_count == 8 { self.buffer.push(self.current_byte); self.current_byte = 0; self.bit_count = 0; }
    }
    fn flush(&mut self) -> Vec<u8> {
        if self.bit_count > 0 { self.buffer.push(self.current_byte); self.current_byte = 0; self.bit_count = 0; }
        self.buffer.clone()
    }
}

struct BitReader { buffer: Vec<u8>, byte_pos: usize, bit_pos: u8 }
impl BitReader {
    fn new(data: Vec<u8>) -> Self { BitReader { buffer: data, byte_pos: 0, bit_pos: 0 } }
    fn read_bit(&mut self) -> u8 {
        if self.byte_pos >= self.buffer.len() { return 0; }
        let bit = (self.buffer[self.byte_pos] >> self.bit_pos) & 1;
        self.bit_pos += 1;
        if self.bit_pos == 8 { self.bit_pos = 0; self.byte_pos += 1; }
        bit
    }
}

struct ArithmeticEncoder { writer: BitWriter, low: u64, high: u64, pending_bits: u32 }
impl ArithmeticEncoder {
    fn new(writer: BitWriter) -> Self { ArithmeticEncoder { writer, low: 0, high: WHOLE, pending_bits: 0 } }
    fn update(&mut self, freq_low: u64, freq_high: u64, total: u64) {
        let range = self.high - self.low + 1;
        self.high = self.low + (range * freq_high) / total - 1;
        self.low = self.low + (range * freq_low) / total;

        loop {
            if self.high < HALF {
                self.writer.write_bit(0);
                for _ in 0..self.pending_bits { self.writer.write_bit(1); }
                self.pending_bits = 0;
            } else if self.low >= HALF {
                self.writer.write_bit(1);
                for _ in 0..self.pending_bits { self.writer.write_bit(0); }
                self.pending_bits = 0;
                self.low -= HALF; self.high -= HALF;
            } else if self.low >= QUARTER && self.high < 3 * QUARTER {
                self.pending_bits += 1;
                self.low -= QUARTER; self.high -= QUARTER;
            } else { break; }

            self.low <<= 1;
            self.high = (self.high << 1) | 1;
        }
    }
    fn finish(mut self) -> Vec<u8> {
        self.pending_bits += 1;
        if self.low < QUARTER {
            self.writer.write_bit(0);
            for _ in 0..self.pending_bits { self.writer.write_bit(1); }
        } else {
            self.writer.write_bit(1);
            for _ in 0..self.pending_bits { self.writer.write_bit(0); }
        }
        self.writer.flush()
    }
}

struct ArithmeticDecoder { reader: BitReader, low: u64, high: u64, code: u64 }
impl ArithmeticDecoder {
    fn new(data: Vec<u8>) -> Self {
        let mut reader = BitReader::new(data);
        let mut code: u64 = 0;
        for _ in 0..PRECISION { code = (code << 1) | reader.read_bit() as u64; }
        ArithmeticDecoder { reader, low: 0, high: WHOLE, code }
    }
    fn get_value(&self, total: u64) -> u64 {
        let range = self.high - self.low + 1;
        ((self.code - self.low + 1) * total - 1) / range
    }
    fn update(&mut self, freq_low: u64, freq_high: u64, total: u64) {
        let range = self.high - self.low + 1;
        self.high = self.low + (range * freq_high) / total - 1;
        self.low = self.low + (range * freq_low) / total;

        loop {
            if self.high < HALF {
                // do nothing
            } else if self.low >= HALF {
                self.code -= HALF; self.low -= HALF; self.high -= HALF;
            } else if self.low >= QUARTER && self.high < 3 * QUARTER {
                self.code -= QUARTER; self.low -= QUARTER; self.high -= QUARTER;
            } else { break; }

            self.low <<= 1;
            self.high = (self.high << 1) | 1;
            self.code = (self.code << 1) | self.reader.read_bit() as u64;
        }
    }
}

// ==========================================
// ADAPTIVE PROBABILITY MODELS (FENWICK TREE)
// ==========================================
#[derive(Copy, Clone)]
struct BitModel { freq0: u32, freq1: u32 }
impl BitModel {
    fn new() -> Self { BitModel { freq0: 1, freq1: 1 } }
    fn get_range(&self, bit: u8) -> (u64, u64, u64) {
        if bit == 0 { (0, self.freq0 as u64, (self.freq0 + self.freq1) as u64) }
        else { (self.freq0 as u64, (self.freq0 + self.freq1) as u64, (self.freq0 + self.freq1) as u64) }
    }
    fn find(&self, value: u64) -> u8 {
        if value < self.freq0 as u64 { 0 } else { 1 }
    }
    fn update(&mut self, bit: u8) {
        if bit == 0 { self.freq0 += 1; } else { self.freq1 += 1; }
        if self.freq0 + self.freq1 > 0xFFFF { self.freq0 = (self.freq0 >> 1) + 1; self.freq1 = (self.freq1 >> 1) + 1; }
    }
}

struct FreqModel {
    tree: Vec<u32>,
    size: usize,
    total: u32,
}
impl FreqModel {
    fn new(size: usize) -> Self {
        let mut model = FreqModel {
            tree: vec![0; size + 1],
            size,
            total: 0,
        };
        for i in 0..size {
            model.update(i, 1);
        }
        model
    }

    fn update(&mut self, mut sym: usize, delta: u32) {
        sym += 1;
        while sym <= self.size {
            self.tree[sym] = self.tree[sym].saturating_add(delta);
            sym += sym & sym.wrapping_neg();
        }
        self.total = self.total.saturating_add(delta);
    }

    fn prefix_sum(&self, mut sym: usize) -> u32 {
        sym += 1;
        let mut sum = 0;
        while sym > 0 {
            sum += self.tree[sym];
            sym -= sym & sym.wrapping_neg();
        }
        sum
    }

    fn get_range(&self, sym: usize) -> (u64, u64, u64) {
        let low = if sym == 0 { 0 } else { self.prefix_sum(sym - 1) };
        let high = self.prefix_sum(sym);
        (low as u64, high as u64, self.total as u64)
    }

    fn find_symbol(&self, mut target: u64) -> usize {
        let mut pos = 0;
        let mut bitmask = 1;
        while (bitmask << 1) <= self.size {
            bitmask <<= 1;
        }

        while bitmask > 0 {
            let next_pos = pos + bitmask;
            if next_pos <= self.size && (self.tree[next_pos] as u64) <= target {
                pos = next_pos;
                target -= self.tree[pos] as u64;
            }
            bitmask >>= 1;
        }
        pos
    }

    fn rescale(&mut self) {
        let mut new_freqs = vec![0u32; self.size];
        for i in 0..self.size {
            let freq = self.get_range(i).1 - self.get_range(i).0;
            new_freqs[i] = (freq >> 1) + 1;
        }

        for i in 0..self.tree.len() {
            self.tree[i] = 0;
        }
        self.total = 0;

        for i in 0..self.size {
            self.update(i, new_freqs[i]);
        }
    }

    fn adaptive_update(&mut self, sym: usize) {
        self.update(sym, 1);
        if self.total > 0xFFFF {
            self.rescale();
        }
    }
}

// ==========================================
// 2-BIT GENOMIC PRE-PROCESSOR
// ==========================================
fn pack_dna(raw: &[u8]) -> Vec<u8> {
    let mut packed = Vec::with_capacity(raw.len() / 4);
    let mut bit_buf: u8 = 0;
    let mut bit_count: u8 = 0;

    for &byte in raw {
        let val = match byte {
            b'A' => 0,
            b'C' => 1,
            b'G' => 2,
            b'T' => 3,
            _ => continue, // Skip newlines and headers
        };
        bit_buf = (bit_buf << 2) | val;
        bit_count += 2;
        if bit_count == 8 {
            packed.push(bit_buf);
            bit_buf = 0;
            bit_count = 0;
        }
    }
    if bit_count > 0 {
        packed.push(bit_buf << (8 - bit_count));
    }
    packed
}

fn unpack_dna(packed: &[u8]) -> Vec<u8> {
    let mut unpacked = Vec::with_capacity(packed.len() * 4);
    let mapping = [b'A', b'C', b'G', b'T'];

    for &byte in packed {
        for i in (0..4).rev() {
            let val = (byte >> (i * 2)) & 0x03;
            unpacked.push(mapping[val as usize]);
        }
    }
    unpacked
}

// ==========================================
// THE 3D ARITHMETIC ENGINE
// ==========================================
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("YK 3D Arithmetic Engine (Genomic Mode)\nUsage:");
        println!("  Compress:   ./yk_engine compress <input_file> <output_file.yk> [reference_file]");
        println!("  Decompress: ./yk_engine decompress <input_file.yk> <output_file> [reference_file]");
        return;
    }

    let command = &args[1];
    let input_file = &args[2];
    let output_file = if args.len() >= 4 { args[3].clone() } else { "output.yk".to_string() };
    let reference_file = if args.len() >= 5 { Some(args[4].clone()) } else { None };

    // Auto-detect Genomic Mode
    let is_genomic = input_file.ends_with(".fna") || input_file.ends_with(".fasta") || input_file.ends_with(".fa");

    match command.as_str() {
        "compress" => {
            let raw_target = fs::read(input_file).expect("Failed to read input file");

            // 2-Bit Pack if Genomic
            let target_data = if is_genomic {
                println!("Genomic mode active: 2-bit packing DNA...");
                pack_dna(&raw_target)
            } else {
                raw_target.clone()
            };

            let raw_ref = if let Some(ref path) = reference_file {
                fs::read(path).expect("Failed to read reference file")
            } else {
                Vec::new()
            };

            let ref_data = if is_genomic && !raw_ref.is_empty() {
                pack_dna(&raw_ref)
            } else {
                raw_ref
            };

            let data = if !ref_data.is_empty() {
                let mut combined = ref_data.clone();
                combined.extend_from_slice(&target_data);
                combined
            } else {
                target_data.clone()
            };

            let target_start_offset = ref_data.len();

            let mut spatial_hash: HashMap<[u8; 4], Vec<usize>> = HashMap::new();
            for i in 0..(data.len().saturating_sub(4)) {
                let mut seq = [0u8; 4];
                seq.copy_from_slice(&data[i..i+4]);
                spatial_hash.entry(seq).or_insert_with(Vec::new).push(i);
            }

            let writer = BitWriter::new();
            let mut enc = ArithmeticEncoder::new(writer);

            let mut sym_model = FreqModel::new(257);
            let mut len_model = FreqModel::new(256);
            let mut dir_model = FreqModel::new(26);
            let mut coord_models_x = [BitModel::new(); 21];
            let mut coord_models_y = [BitModel::new(); 21];
            let mut coord_models_z = [BitModel::new(); 21];

            // Header: [Target Length (4)] [Ref Offset (4)] [Genomic Flag (1)]
            let mut final_output = (target_data.len() as u32).to_le_bytes().to_vec();
            final_output.extend_from_slice(&(target_start_offset as u32).to_le_bytes());
            final_output.push(if is_genomic { 1 } else { 0 });

            let mut i = target_start_offset;
            while i < data.len() {
                if i % 100000 == 0 && i != target_start_offset {
                    println!("Processed {} / {} bytes...", i - target_start_offset, target_data.len());
                }

                let mut best_len = 0;
                let mut best_off = 0;
                let mut best_dir = 0;

                if i + 4 <= data.len() {
                    let mut seq = [0u8; 4];
                    seq.copy_from_slice(&data[i..i+4]);
                    if let Some(positions) = spatial_hash.get(&seq) {
                        for &p in positions.iter().rev().take(50) {
                            if p >= i { continue; }
                            let (ax, ay, az) = offset_to_3d(p);

                            for (dir_idx, &(dx, dy, dz)) in DIRECTIONS.iter().enumerate() {
                                let mut len = 0;
                                let mut curr_x = ax as i32;
                                let mut curr_y = ay as i32;
                                let mut curr_z = az as i32;

                                while i + len < data.len() && len < 259 {
                                    if curr_x < 0 || curr_y < 0 || curr_z < 0 { break; }
                                    let grid_idx = to_3d_offset(curr_x as u32, curr_y as u32, curr_z as u32);
                                    if grid_idx >= data.len() || grid_idx >= i + len { break; }
                                    if data[i + len] != data[grid_idx] { break; }

                                    len += 1;
                                    curr_x += dx; curr_y += dy; curr_z += dz;
                                }

                                if len > best_len { best_len = len; best_off = p; best_dir = dir_idx; }
                            }
                        }
                    }
                }

                if best_len >= 4 {
                    let (l, h, t) = sym_model.get_range(256);
                    enc.update(l, h, t);
                    sym_model.adaptive_update(256);

                    let len_sym = (best_len - 4).min(255);
                    let (l, h, t) = len_model.get_range(len_sym);
                    enc.update(l, h, t);
                    len_model.adaptive_update(len_sym);

                    let (l, h, t) = dir_model.get_range(best_dir);
                    enc.update(l, h, t);
                    dir_model.adaptive_update(best_dir);

                    let (x, y, z) = offset_to_3d(best_off);
                    let coords = [x, y, z];
                    for axis in 0..3 {
                        for bit in (0..21).rev() {
                            let b = ((coords[axis] >> bit) & 1) as u8;

                            if axis == 0 {
                                let (fl, fh, ft) = coord_models_x[bit].get_range(b);
                                enc.update(fl, fh, ft);
                                coord_models_x[bit].update(b);
                            } else if axis == 1 {
                                let (fl, fh, ft) = coord_models_y[bit].get_range(b);
                                enc.update(fl, fh, ft);
                                coord_models_y[bit].update(b);
                            } else {
                                let (fl, fh, ft) = coord_models_z[bit].get_range(b);
                                enc.update(fl, fh, ft);
                                coord_models_z[bit].update(b);
                            }
                        }
                    }
                    i += best_len;
                } else {
                    let (l, h, t) = sym_model.get_range(data[i] as usize);
                    enc.update(l, h, t);
                    sym_model.adaptive_update(data[i] as usize);
                    i += 1;
                }
            }

            let compressed = enc.finish();
            final_output.extend(compressed);

            fs::write(&output_file, &final_output).expect("Failed to write");
            println!("Success! 3D Arithmetic Compressed {} ({} bytes) -> {} ({} bytes)", input_file, target_data.len(), output_file, final_output.len());
        }
        "decompress" => {
            let data = fs::read(input_file).expect("Failed to read input file");
            if data.len() < 9 { panic!("Invalid YK file"); }

            let bytes_to_decode = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
            let target_start_offset = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
            let is_genomic = data[8] == 1;

            let mut dec = ArithmeticDecoder::new(data[9..].to_vec());

            let mut sym_model = FreqModel::new(257);
            let mut len_model = FreqModel::new(256);
            let mut dir_model = FreqModel::new(26);
            let mut coord_models_x = [BitModel::new(); 21];
            let mut coord_models_y = [BitModel::new(); 21];
            let mut coord_models_z = [BitModel::new(); 21];

            let mut out: Vec<u8> = if let Some(ref path) = reference_file {
                let raw_ref = fs::read(path).expect("Failed to read reference file");
                if is_genomic {
                    pack_dna(&raw_ref)
                } else {
                    raw_ref
                }
            } else {
                Vec::new()
            };

            if out.len() != target_start_offset {
                println!("Warning: Reference file size mismatch! Decompression may fail.");
            }

            while out.len() < bytes_to_decode + target_start_offset {
                let total = sym_model.total as u64;
                let val = dec.get_value(total);
                let sym = sym_model.find_symbol(val);

                let (l, h, _) = sym_model.get_range(sym);
                dec.update(l, h, total);
                sym_model.adaptive_update(sym);

                if sym == 256 {
                    let len_total = len_model.total as u64;
                    let len_val = dec.get_value(len_total);
                    let len_sym = len_model.find_symbol(len_val);
                    let (l, h, _) = len_model.get_range(len_sym);
                    dec.update(l, h, len_total);
                    len_model.adaptive_update(len_sym);
                    let len = len_sym + 4;

                    let dir_total = dir_model.total as u64;
                    let dir_val = dec.get_value(dir_total);
                    let dir_idx = dir_model.find_symbol(dir_val);
                    let (l, h, _) = dir_model.get_range(dir_idx);
                    dec.update(l, h, dir_total);
                    dir_model.adaptive_update(dir_idx);

                    let mut coords = [0u32; 3];
                    for axis in 0..3 {
                        for bit in (0..21).rev() {
                            let b_total = if axis == 0 {
                                (coord_models_x[bit].freq0 + coord_models_x[bit].freq1) as u64
                            } else if axis == 1 {
                                (coord_models_y[bit].freq0 + coord_models_y[bit].freq1) as u64
                            } else {
                                (coord_models_z[bit].freq0 + coord_models_z[bit].freq1) as u64
                            };

                            let b_val = dec.get_value(b_total);
                            let b = if axis == 0 {
                                coord_models_x[bit].find(b_val)
                            } else if axis == 1 {
                                coord_models_y[bit].find(b_val)
                            } else {
                                coord_models_z[bit].find(b_val)
                            };

                            let (fl, fh, _) = if axis == 0 {
                                coord_models_x[bit].get_range(b)
                            } else if axis == 1 {
                                coord_models_y[bit].get_range(b)
                            } else {
                                coord_models_z[bit].get_range(b)
                            };

                            dec.update(fl, fh, b_total);

                            if axis == 0 { coord_models_x[bit].update(b); }
                            else if axis == 1 { coord_models_y[bit].update(b); }
                            else { coord_models_z[bit].update(b); }

                            if b == 1 { coords[axis] |= 1 << bit; }
                        }
                    }

                    let (dx, dy, dz) = DIRECTIONS[dir_idx];
                    let mut curr_x = coords[0] as i32;
                    let mut curr_y = coords[1] as i32;
                    let mut curr_z = coords[2] as i32;

                    for _ in 0..len {
                        let grid_idx = to_3d_offset(curr_x as u32, curr_y as u32, curr_z as u32);
                        out.push(out[grid_idx]);
                        curr_x += dx; curr_y += dy; curr_z += dz;
                    }
                } else {
                    out.push(sym as u8);
                }
            }

            // Strip reference and unpack if genomic
            let packed_data = &out[target_start_offset..];
            let final_data = if is_genomic {
                println!("Genomic mode active: Unpacking 2-bit DNA...");
                unpack_dna(packed_data)
            } else {
                packed_data.to_vec()
            };

            fs::write(&output_file, &final_data).expect("Failed to write");
            println!("Success! 3D Arithmetic Decompressed {} -> {} ({} bytes)", input_file, output_file, final_data.len());
        }
        _ => println!("Unknown command."),
    }
}
