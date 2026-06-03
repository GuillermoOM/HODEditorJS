fn main() {
    let rgba = vec![0u8; 16*4];
    let mut out = vec![0u8; 8];
    texpresso::Format::Bc1.compress(&rgba, 4, 4, texpresso::Params::default(), &mut out);
    println!("out: {}", out.len());
}
