use hwr_hod_parser::xpress;

fn main() {
    let mut data = Vec::new();
    for _ in 0..1000 {
        data.extend_from_slice(b"Hello World!");
    }
    let comp = xpress::compress(&data);
    println!("Uncompressed: {}", data.len());
    println!("Compressed: {}", comp.len());
}
