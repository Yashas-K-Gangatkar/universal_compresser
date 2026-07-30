use std::fs;
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::env;

// ==========================================
// BIT WRITER & READER
// ==========================================
struct BitWriter { buffer: Vec<u8>, current_byte: u8, bit_count: u8 }
impl BitWriter {
    fn new() -> Self { BitWriter { buffer: Vec::new(), current_byte: 0, bit_count: 0 } }
    fn write_bits_msb(&mut self, value: u32, bits: u8) {
        for i in (0..bits).rev() {
            self.current_byte |= (((value >> i) & 1) as u8) << self.bit_count;
            self.bit_count += 1;
            if self.bit_count == 8 { self.buffer.push(self.current_byte); self.current_byte = 0; self.bit_count = 0; }
        }
    }
    fn write_bits_lsb(&mut self, value: u32, bits: u8) {
        for i in 0..bits {
            self.current_byte |= (((value >> i) & 1) as u8) << self.bit_count;
            self.bit_count += 1;
            if self.bit_count == 8 { self.buffer.push(self.current_byte); self.current_byte = 0; self.bit_count = 0; }
        }
    }
    fn flush(&mut self) -> Vec<u8> {
        if self.bit_count > 0 { self.buffer.push(self.current_byte); self.current_byte = 0; self.bit_count = 0; }
        self.buffer.clone()
    }
}

struct BitReader { data: Vec<u8>, byte_pos: usize, bit_pos: u8 }
impl BitReader {
    fn new(data: Vec<u8>) -> Self { BitReader { data, byte_pos: 0, bit_pos: 0 } }
    fn read_bit(&mut self) -> Option<u8> {
        if self.byte_pos >= self.data.len() { return None; }
        let bit = (self.data[self.byte_pos] >> self.bit_pos) & 1;
        self.bit_pos += 1;
        if self.bit_pos == 8 { self.bit_pos = 0; self.byte_pos += 1; }
        Some(bit)
    }
    fn read_bits_lsb(&mut self, bits: u8) -> Option<u32> {
        let mut val = 0;
        for i in 0..bits { val |= (self.read_bit()? as u32) << i; }
        Some(val)
    }
}

// ==========================================
// HUFFMAN TREE & CODES (Encoder)
// ==========================================
#[derive(Clone)]
struct Node { freq: u32, sym: u16, left: Option<Box<Node>>, right: Option<Box<Node>> }
impl Eq for Node {}
impl PartialEq for Node { fn eq(&self, o: &Self) -> bool { self.freq == o.freq } }
impl PartialOrd for Node { fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) } }
impl Ord for Node { fn cmp(&self, o: &Self) -> Ordering { o.freq.cmp(&self.freq) } }

fn build_lengths(freq: &[u32], max_bits: usize) -> Vec<u8> {
    let mut heap = BinaryHeap::new();
    for i in 0..freq.len() { if freq[i] > 0 { heap.push(Node { freq: freq[i], sym: i as u16, left: None, right: None }); } }

    let mut lengths = vec![0u8; freq.len()];
    if heap.is_empty() { return lengths; }
    if heap.len() == 1 {
        let sym = heap.pop().unwrap().sym as usize;
        lengths[sym] = 1;
        for i in 0..lengths.len() {
            if i != sym && freq[i] == 0 { lengths[i] = 1; break; }
        }
        return lengths;
    }

    while heap.len() > 1 {
        let l = heap.pop().unwrap(); let r = heap.pop().unwrap();
        heap.push(Node { freq: l.freq + r.freq, sym: 0, left: Some(Box::new(l)), right: Some(Box::new(r)) });
    }
    let root = heap.pop().unwrap();

    let mut stack = vec![(root, 0u32)];
    while let Some((node, depth)) = stack.pop() {
        if node.left.is_none() && node.right.is_none() { lengths[node.sym as usize] = depth as u8; }
        else {
            if let Some(l) = node.left { stack.push((*l, depth + 1)); }
            if let Some(r) = node.right { stack.push((*r, depth + 1)); }
        }
    }

    let mut bl_count = vec![0u32; 40];
    for &l in &lengths { if l > 0 && l < 40 { bl_count[l as usize] += 1; } }

    // 1. Move all codes longer than max_bits to max_bits
    for l in (max_bits + 1)..40 {
        bl_count[max_bits] += bl_count[l];
        bl_count[l] = 0;
    }

    // 2. THE FIX: Perfectly balance the tree to satisfy Kraft inequality
    // Calculate current Kraft sum
    let mut total = 0u64;
    for i in 1..=max_bits { total += (bl_count[i] as u64) << (max_bits - i); }

    // Fix over-subscription (Kraft sum > 2^max_bits)
    while total > (1u64 << max_bits) {
        let mut bits = max_bits - 1;
        while bl_count[bits] == 0 { bits -= 1; }
        bl_count[bits] -= 1;
        bl_count[bits + 1] += 2;
        bl_count[max_bits] -= 1;
        total = 0;
        for i in 1..=max_bits { total += (bl_count[i] as u64) << (max_bits - i); }
    }

    // Fix under-subscription (Kraft sum < 2^max_bits)
    while total < (1u64 << max_bits) {
        let mut bits = max_bits;
        while bl_count[bits] == 0 { bits -= 1; }
        if bits > 1 {
            bl_count[bits] -= 1;
            bl_count[bits - 1] += 1;
            total = 0;
            for i in 1..=max_bits { total += (bl_count[i] as u64) << (max_bits - i); }
        } else { break; }
    }

    // 3. Reassign lengths to symbols sorted by frequency (highest first)
    let mut symbols: Vec<usize> = (0..freq.len()).filter(|&i| lengths[i] > 0).collect();
    symbols.sort_by(|&a, &b| freq[b].cmp(&freq[a]).then(a.cmp(&b)));

    let mut new_lengths = vec![0u8; freq.len()];
    let mut sym_idx = 0;
    for l in 1..=max_bits {
        for _ in 0..bl_count[l] {
            if sym_idx < symbols.len() {
                new_lengths[symbols[sym_idx]] = l as u8;
                sym_idx += 1;
            }
        }
    }
    new_lengths
}

fn gen_canonical_codes(lengths: &[u8]) -> Vec<u32> {
    let mut bl_count = [0u32; 16];
    for &l in lengths { if l > 0 { bl_count[l as usize] += 1; } }
    let mut next_code = [0u32; 16];
    let mut code = 0;
    for bits in 1..16 { code = (code + bl_count[bits - 1]) << 1; next_code[bits] = code; }
    let mut codes = vec![0u32; lengths.len()];
    for i in 0..lengths.len() {
        let l = lengths[i];
        if l > 0 { codes[i] = next_code[l as usize]; next_code[l as usize] += 1; }
    }
    codes
}

// ==========================================
// HUFFMAN TREE (Decoder)
// ==========================================
struct SimpleHuffmanTree { codes: Vec<u32>, lengths: Vec<u8> }
impl SimpleHuffmanTree {
    fn build(lengths: &[u8]) -> Self {
        let codes = gen_canonical_codes(lengths);
        SimpleHuffmanTree { codes, lengths: lengths.to_vec() }
    }
    fn decode(&self, reader: &mut BitReader) -> Option<usize> {
        let mut code: u32 = 0;
        let mut len: u8 = 0;
        loop {
            code = (code << 1) | reader.read_bit()? as u32;
            len += 1;
            if len > 15 { return None; }
            for i in 0..self.lengths.len() {
                if self.lengths[i] == len && self.codes[i] == code { return Some(i); }
            }
        }
    }
}

// ==========================================
// DEFLATE LOOKUP TABLES
// ==========================================
fn get_length_symbol(len: usize) -> (u16, u8, u16) {
    match len {
        3 => (257, 0, 3), 4 => (258, 0, 4), 5 => (259, 0, 5), 6 => (260, 0, 6),
        7 => (261, 0, 7), 8 => (262, 0, 8), 9 => (263, 0, 9), 10 => (264, 0, 10),
        11..=12 => (265, 1, 11), 13..=14 => (266, 1, 13), 15..=16 => (267, 1, 15),
        17..=18 => (268, 1, 17), 19..=22 => (269, 2, 19), 23..=26 => (270, 2, 23),
        27..=30 => (271, 2, 27), 31..=34 => (272, 2, 31), 35..=42 => (273, 3, 35),
        43..=50 => (274, 3, 43), 51..=58 => (275, 3, 51), 59..=66 => (276, 3, 59),
        67..=82 => (277, 4, 67), 83..=98 => (278, 4, 83), 99..=114 => (279, 4, 99),
        115..=130 => (280, 4, 115), 131..=162 => (281, 5, 131), 163..=194 => (282, 5, 163),
        195..=226 => (283, 5, 195), 227..=257 => (284, 5, 227), 258 => (285, 0, 258),
        _ => panic!("Invalid length"),
    }
}

fn get_distance_symbol(dist: usize) -> (u8, u8, u16) {
    match dist {
        1 => (0, 0, 1), 2 => (1, 0, 2), 3 => (2, 0, 3), 4 => (3, 0, 4),
        5..=6 => (4, 1, 5), 7..=8 => (5, 1, 7), 9..=12 => (6, 2, 9),
        13..=16 => (7, 2, 13), 17..=24 => (8, 3, 17), 25..=32 => (9, 3, 25),
        33..=48 => (10, 4, 33), 49..=64 => (11, 4, 49), 65..=96 => (12, 5, 65),
        97..=128 => (13, 5, 97), 129..=192 => (14, 6, 129), 193..=256 => (15, 6, 193),
        257..=384 => (16, 7, 257), 385..=512 => (17, 7, 385), 513..=768 => (18, 8, 513),
        769..=1024 => (19, 8, 769), 1025..=1536 => (20, 9, 1025), 1537..=2048 => (21, 9, 1537),
        2049..=3072 => (22, 10, 2049), 3073..=4096 => (23, 10, 3073), 4097..=6144 => (24, 11, 4097),
        6145..=8192 => (25, 11, 6145), 8193..=12288 => (26, 12, 8193), 12289..=16384 => (27, 12, 12289),
        16385..=24576 => (28, 13, 16385), 24577..=32768 => (29, 13, 24577),
        _ => panic!("Invalid distance"),
    }
}

fn get_length_base_and_extra(sym: usize) -> (usize, u8) {
    match sym {
        257 => (3, 0), 258 => (4, 0), 259 => (5, 0), 260 => (6, 0), 261 => (7, 0), 262 => (8, 0), 263 => (9, 0), 264 => (10, 0),
        265 => (11, 1), 266 => (13, 1), 267 => (15, 1), 268 => (17, 1), 269 => (19, 2), 270 => (23, 2), 271 => (27, 2), 272 => (31, 2),
        273 => (35, 3), 274 => (43, 3), 275 => (51, 3), 276 => (59, 3), 277 => (67, 4), 278 => (83, 4), 279 => (99, 4), 280 => (115, 4),
        281 => (131, 5), 282 => (163, 5), 283 => (195, 5), 284 => (227, 5), 285 => (258, 0), _ => panic!("Invalid length symbol"),
    }
}

fn get_distance_base_and_extra(sym: usize) -> (usize, u8) {
    match sym {
        0 => (1, 0), 1 => (2, 0), 2 => (3, 0), 3 => (4, 0), 4 => (5, 1), 5 => (7, 1), 6 => (9, 2), 7 => (13, 2),
        8 => (17, 3), 9 => (25, 3), 10 => (33, 4), 11 => (49, 4), 12 => (65, 5), 13 => (97, 5), 14 => (129, 6), 15 => (193, 6),
        16 => (257, 7), 17 => (385, 7), 18 => (513, 8), 19 => (769, 8), 20 => (1025, 9), 21 => (1537, 9), 22 => (2049, 10), 23 => (3073, 10),
        24 => (4097, 11), 25 => (6145, 11), 26 => (8193, 12), 27 => (12289, 12), 28 => (16385, 13), 29 => (24577, 13), _ => panic!("Invalid distance symbol"),
    }
}

// ==========================================
// COMPRESS LOGIC (WITH EXTERNAL MASTER GRID)
// ==========================================
fn compress_file(input_path: &str, output_path: &str) {
    let data = fs::read(input_path).expect("Failed to read input file");
    let mut bw = BitWriter::new();

    // 1. Load the External Master Grid (Sherlock Holmes)
    let dict_path = "/Users/yashas/Desktop/YK_Algorithm/master_dict.txt";
    let dict_data = fs::read(dict_path).expect("Failed to read Master Dictionary");

    // DEFLATE window is 32KB. We take the last 32KB of Sherlock Holmes.
    let dict_size = 32768;
    let preset_dict = if dict_data.len() > dict_size {
        &dict_data[dict_data.len() - dict_size..]
    } else {
        &dict_data[..]
    };

    // Create the unified window: [Preset Dictionary] + [Alice in Wonderland]
    let mut full_window = preset_dict.to_vec();
    full_window.extend_from_slice(&data);

    let mut hash_map: HashMap<[u8; 4], Vec<usize>> = HashMap::new();

    // Pre-seed hash map with the Master Grid
    for i in 0..(preset_dict.len().saturating_sub(4)) {
        let mut seq = [0u8; 4];
        seq.copy_from_slice(&full_window[i..i+4]);
        hash_map.entry(seq).or_insert_with(Vec::new).push(i);
    }

    let mut tokens: Vec<(u16, u16, u8, u32, u8, u32)> = Vec::new();
    let mut lit_freq = [0u32; 286];
    let mut dist_freq = [0u32; 30];
    lit_freq[256] = 1;

    let mut i = preset_dict.len(); // Start processing AFTER the dictionary
    let total = full_window.len();

    while i < total {
        let mut best_len = 0;
        let mut best_dist = 0;
        if i + 4 <= total {
            let mut seq = [0u8; 4]; seq.copy_from_slice(&full_window[i..i+4]);
            if let Some(pos) = hash_map.get(&seq) {
                for &p in pos.iter().rev().take(100) {
                    let d = i - p; if d > 32768 { continue; }
                    let mut l = 3; let max_l = std::cmp::min(total - i, 258);
                    while l < max_l && full_window[i+l] == full_window[p+l] { l += 1; }
                    if l > best_len { best_len = l; best_dist = d; }
                }
            }
        }

        let mut lazy_len = 0;
        if best_len > 0 && best_len < 258 && i + 5 <= total {
            let mut l_seq = [0u8; 4]; l_seq.copy_from_slice(&full_window[i+1..i+5]);
            if let Some(pos) = hash_map.get(&l_seq) {
                for &p in pos.iter().rev().take(100) {
                    let d = (i + 1) - p; if d > 32768 { continue; }
                    let mut l = 3; let max_l = std::cmp::min(total - (i + 1), 258);
                    while l < max_l && full_window[(i+1)+l] == full_window[p+l] { l += 1; }
                    if l > lazy_len { lazy_len = l; }
                }
            }
        }

        if best_len > 0 && lazy_len > best_len {
            let b = full_window[i] as u16; tokens.push((b, 0, 0, 0, 0, 0)); lit_freq[b as usize] += 1;
            if i + 4 <= total { let mut s = [0u8; 4]; s.copy_from_slice(&full_window[i..i+4]); hash_map.entry(s).or_insert_with(Vec::new).push(i); }
            i += 1;
        } else if best_len >= 3 {
            let (lsym, lextra, lbase) = get_length_symbol(best_len);
            let (dsym, dextra, dbase) = get_distance_symbol(best_dist);
            let lval = (best_len - lbase as usize) as u32;
            let dval = (best_dist - dbase as usize) as u32;
            tokens.push((lsym, dsym as u16, lextra, lval, dextra, dval));
            lit_freq[lsym as usize] += 1; dist_freq[dsym as usize] += 1;
            for k in 0..best_len { if i + k + 4 <= total { let mut s = [0u8; 4]; s.copy_from_slice(&full_window[i+k..i+k+4]); hash_map.entry(s).or_insert_with(Vec::new).push(i + k); } }
            i += best_len;
        } else {
            let b = full_window[i] as u16; tokens.push((b, 0, 0, 0, 0, 0)); lit_freq[b as usize] += 1;
            if i + 4 <= total { let mut s = [0u8; 4]; s.copy_from_slice(&full_window[i..i+4]); hash_map.entry(s).or_insert_with(Vec::new).push(i); }
            i += 1;
        }
    }

    if dist_freq.iter().all(|&f| f == 0) { dist_freq[0] = 1; }

    let lit_lengths = build_lengths(&lit_freq, 15); let lit_codes = gen_canonical_codes(&lit_lengths);
    let dist_lengths = build_lengths(&dist_freq, 15); let dist_codes = gen_canonical_codes(&dist_lengths);

    let mut hlit = 257; for j in (0..286).rev() { if lit_freq[j] > 0 { hlit = j + 1; break; } }
    let mut hdist = 1; for j in (0..30).rev() { if dist_freq[j] > 0 { hdist = j + 1; break; } }
    let actual_lengths: Vec<u8> = lit_lengths[..hlit].iter().chain(dist_lengths[..hdist].iter()).cloned().collect();

    let mut rle_tokens: Vec<(u8, u8, u32)> = Vec::new(); let mut cl_freq = [0u32; 19];
    let mut idx = 0;
    while idx < actual_lengths.len() {
        let len = actual_lengths[idx];
        if len == 0 {
            let mut count = 1; while idx + count < actual_lengths.len() && actual_lengths[idx + count] == 0 && count < 138 { count += 1; }
            if count >= 11 { rle_tokens.push((18, 7, count as u32 - 11)); cl_freq[18] += 1; idx += count; }
            else if count >= 3 { rle_tokens.push((17, 3, count as u32 - 3)); cl_freq[17] += 1; idx += count; }
            else { rle_tokens.push((0, 0, 0)); cl_freq[0] += 1; idx += 1; }
        } else {
            rle_tokens.push((len, 0, 0)); cl_freq[len as usize] += 1; idx += 1;
            if idx < actual_lengths.len() && actual_lengths[idx] == len {
                let mut count = 1; while idx + count < actual_lengths.len() && actual_lengths[idx + count] == len && count < 6 { count += 1; }
                if count >= 3 { rle_tokens.push((16, 2, count as u32 - 3)); cl_freq[16] += 1; idx += count; }
            }
        }
    }

    let cl_lengths = build_lengths(&cl_freq, 7); let cl_codes = gen_canonical_codes(&cl_lengths);

    bw.write_bits_lsb(1, 1); bw.write_bits_lsb(2, 2);
    bw.write_bits_lsb(hlit as u32 - 257, 5); bw.write_bits_lsb(hdist as u32 - 1, 5);
    let hclen_order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let mut hclen = 4; for j in (0..19).rev() { if cl_lengths[hclen_order[j]] > 0 { hclen = j + 1; break; } }
    bw.write_bits_lsb(hclen as u32 - 4, 4);
    for j in 0..hclen { bw.write_bits_lsb(cl_lengths[hclen_order[j]] as u32, 3); }
    for (sym, extra_bits, extra_val) in &rle_tokens { let l = cl_lengths[*sym as usize]; let c = cl_codes[*sym as usize]; bw.write_bits_msb(c, l); if *extra_bits > 0 { bw.write_bits_lsb(*extra_val, *extra_bits); } }
    for (lsym, dsym, lextra, lval, dextra, dval) in &tokens {
        let l = lit_lengths[*lsym as usize]; let c = lit_codes[*lsym as usize]; bw.write_bits_msb(c, l);
        if *lextra > 0 { bw.write_bits_lsb(*lval, *lextra); }
        if *lsym > 256 { let dl = dist_lengths[*dsym as usize]; let dc = dist_codes[*dsym as usize]; bw.write_bits_msb(dc, dl); if *dextra > 0 { bw.write_bits_lsb(*dval, *dextra); } }
    }
    let eob_l = lit_lengths[256]; let eob_c = lit_codes[256]; bw.write_bits_msb(eob_c, eob_l);

    let final_buffer = bw.flush();
    fs::write(output_path, &final_buffer).expect("Failed to write output file");
    println!("Success! Compressed {} ({} bytes) -> {} ({} bytes)", input_path, data.len(), output_path, final_buffer.len());
}

// ==========================================
// DECOMPRESS LOGIC (WITH EXTERNAL MASTER GRID)
// ==========================================
fn decompress_file(input_path: &str, output_path: &str) {
    let compressed_data = fs::read(input_path).expect("Failed to read .yk file");
    let mut reader = BitReader::new(compressed_data);

    // 1. Load the exact same Master Grid
    let dict_path = "/Users/yashas/Desktop/YK_Algorithm/master_dict.txt";
    let dict_data = fs::read(dict_path).expect("Failed to read Master Dictionary");
    let dict_size = 32768;
    let preset_dict = if dict_data.len() > dict_size {
        &dict_data[dict_data.len() - dict_size..]
    } else {
        &dict_data[..]
    };

    let mut out: Vec<u8> = preset_dict.to_vec(); // Prepend dictionary to output buffer!

    let _bfinal = reader.read_bit().unwrap_or(1);
    let btype = reader.read_bits_lsb(2).unwrap_or(0);

    if btype == 2 { // Dynamic Huffman
        let hlit = reader.read_bits_lsb(5).unwrap() as usize + 257;
        let hdist = reader.read_bits_lsb(5).unwrap() as usize + 1;
        let hclen = reader.read_bits_lsb(4).unwrap() as usize + 4;

        let hclen_order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
        let mut cl_lengths = [0u8; 19];
        for i in 0..hclen { cl_lengths[hclen_order[i]] = reader.read_bits_lsb(3).unwrap() as u8; }

        let cl_tree = SimpleHuffmanTree::build(&cl_lengths);

        let mut lengths = vec![0u8; hlit + hdist];
        let mut i = 0;
        while i < lengths.len() {
            let sym = cl_tree.decode(&mut reader).unwrap();
            if sym < 16 { lengths[i] = sym as u8; i += 1; }
            else if sym == 16 {
                let repeat = reader.read_bits_lsb(2).unwrap() as usize + 3;
                let prev = if i > 0 { lengths[i - 1] } else { 0 };
                for _ in 0..repeat { lengths[i] = prev; i += 1; }
            } else if sym == 17 { i += reader.read_bits_lsb(3).unwrap() as usize + 3; }
            else if sym == 18 { i += reader.read_bits_lsb(7).unwrap() as usize + 11; }
        }

        let lit_tree = SimpleHuffmanTree::build(&lengths[..hlit]);
        let dist_tree = SimpleHuffmanTree::build(&lengths[hlit..]);

        loop {
            let sym = lit_tree.decode(&mut reader).unwrap();
            if sym < 256 { out.push(sym as u8); }
            else if sym == 256 { break; }
            else {
                let (base_len, extra_bits) = get_length_base_and_extra(sym);
                let mut len = base_len;
                if extra_bits > 0 { len += reader.read_bits_lsb(extra_bits).unwrap() as usize; }

                let dist_sym = dist_tree.decode(&mut reader).unwrap();
                let (base_dist, dist_extra_bits) = get_distance_base_and_extra(dist_sym);
                let mut dist = base_dist;
                if dist_extra_bits > 0 { dist += reader.read_bits_lsb(dist_extra_bits).unwrap() as usize; }

                let start = out.len() - dist;
                for j in 0..len { let b = out[start + j]; out.push(b); }
            }
        }
    } else { panic!("Unsupported block type"); }

    // 2. Strip the 32KB dictionary from the output before saving!
    let final_data = &out[preset_dict.len()..];
    fs::write(output_path, final_data).expect("Failed to write output file");
    println!("Success! Decompressed {} -> {} ({} bytes)", input_path, output_path, final_data.len());
}

// ==========================================
// CLI MAIN EXECUTION
// ==========================================
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("YK Universal Compiler (External Dictionary Mode)\nUsage:");
        println!("  Compress:   ./yk_engine compress <input_file> <output_file.yk>");
        println!("  Decompress: ./yk_engine decompress <input_file.yk> <output_file>");
        return;
    }

    let command = &args[1];
    let input_file = &args[2];

    let output_file = if args.len() == 4 {
        args[3].clone()
    } else {
        if command == "compress" { format!("{}.yk", input_file) } else { "decoded_file.txt".to_string() }
    };

    match command.as_str() {
        "compress" => compress_file(input_file, &output_file),
        "decompress" => decompress_file(input_file, &output_file),
        _ => println!("Unknown command. Use 'compress' or 'decompress'."),
    }
}
