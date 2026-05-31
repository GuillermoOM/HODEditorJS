use hwr_hod_parser::hod::HODModel;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let bytes = std::fs::read(&args[1]).unwrap();
    let model = HODModel::parse(&bytes).unwrap();
    for chunk in &model.preserved_chunks {
        println!("Preserved chunk: {}", chunk.id);
        if chunk.id == "INFO" {
            println!("INFO data len: {}", chunk.data.len());
            let mut hex = String::new();
            for b in &chunk.data {
                hex.push_str(&format!("{:02x} ", b));
            }
            println!("{}", hex);
            
            for child in &chunk.children {
                println!("  Child: {} len: {}", child.id, child.data.len());
                let mut chex = String::new();
                for b in &child.data {
                    chex.push_str(&format!("{:02x} ", b));
                }
                println!("  {}", chex);
            }
        }
    }
}
