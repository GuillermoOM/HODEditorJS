use hwr_hod_parser::iff::parse_chunks;
fn main() {
    let bytes = std::fs::read("../testing/ter_centaur/ter_centaur_hodor.hod").unwrap();
    let chunks = parse_chunks(&bytes).unwrap();
    let dtrm = chunks.iter().find(|c| c.id == "DTRM").unwrap();
    let cold = dtrm.children.iter().find(|c| c.id == "COLD").unwrap();
    println!("COLD type: {:?}", cold.chunk_type);
    println!("COLD children count: {}", cold.children.len());
    println!("COLD data length: {}", cold.data.len());
    if !cold.children.is_empty() {
        for child in &cold.children {
            println!("  Child: {} type: {:?} size: {}", child.id, child.chunk_type, child.data.len());
        }
    }
}
