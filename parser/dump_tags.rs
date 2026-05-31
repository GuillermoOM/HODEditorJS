use hwr_hod_parser::iff::parse_chunks;
fn main() {
    let bytes = std::fs::read("../testing/ter_centaur/ter_centaur_hodor.hod").unwrap();
    let chunks = parse_chunks(&bytes).unwrap();
    let dtrm = chunks.iter().find(|c| c.id == "HVMD").unwrap();
    let mult = dtrm.children.iter().find(|c| c.id == "MULT").unwrap();
    let tags = mult.children.iter().find(|c| c.id == "TAGS").unwrap();
    println!("TAGS len: {}, data: {:?}", tags.data.len(), tags.data);
}
