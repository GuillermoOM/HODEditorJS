use hwr_hod_parser::xpress;

fn test_roundtrip(data: &[u8], label: &str) {
    let comp = xpress::compress(data);
    let decomp = xpress::decompress(&comp, data.len()).unwrap();
    if data == decomp.as_slice() {
        println!("PASS: {} ({} bytes -> {} bytes)", label, data.len(), comp.len());
    } else {
        let mut diffs = 0;
        for i in 0..data.len().min(decomp.len()) {
            if data[i] != decomp[i] { diffs += 1; }
        }
        for i in 0..data.len().min(decomp.len()) {
            if data[i] != decomp[i] {
                println!("FAIL: {} — {} diffs, first at byte {}: expected=0x{:02x} got=0x{:02x}", label, diffs, i, data[i], decomp[i]);
                return;
            }
        }
    }
}

fn main() {
    // Small synthetic tests
    test_roundtrip(&vec![0xAA; 100], "100 bytes of 0xAA");
    test_roundtrip(&vec![0xAA; 1000], "1000 bytes of 0xAA");
    test_roundtrip(&vec![0xAA; 4096], "4096 bytes of 0xAA");
    test_roundtrip(&vec![0xAA; 8192], "8192 bytes of 0xAA");

    // Pattern that might trigger the bug
    let mut pattern = Vec::new();
    for i in 0..87424 {
        pattern.push((i % 256) as u8);
    }
    test_roundtrip(&pattern, "87424 bytes sequential");

    // Random-ish data
    let mut data = Vec::new();
    let mut state = 12345u32;
    for _ in 0..87424 {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        data.push((state >> 16) as u8);
    }
    test_roundtrip(&data, "87424 bytes pseudo-random");

    // Actual texture data from HODOR
    let hodor_decomp = std::fs::read("../testing/ter_centaur/rtl_test/decomp_tex.bin").unwrap_or_default();
    if !hodor_decomp.is_empty() {
        test_roundtrip(&hodor_decomp, "HODOR texture pool");
    }

    // Test: compress first N bytes of texture data
    for size in [512, 1024, 1536, 2048, 2560, 3072, 3584, 4096] {
        if size <= hodor_decomp.len() {
            test_roundtrip(&hodor_decomp[0..size], &format!("tex[0..{}]", size));
        }
    }
}
