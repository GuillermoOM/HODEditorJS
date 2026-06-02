use hwr_hod_parser::iff::parse_iff;
use std::fs;

fn main() {
    let bytes = fs::read("../testing/ter_zephyrus/ter_zephyrus_1.0_original.hod").unwrap();
    let iff = parse_iff(&bytes).unwrap();
    for c in &iff.chunks {
        if c.id == "HVMD" {
            println!("HVMD children:");
            for child in &c.children {
                println!("  {}", child.id);
            }
        } else if c.id == "FORM" {
            println!("FORM children:");
            for child in &c.children {
                println!("  {}", child.id);
            }
        }
    }
}
