use std::env;
use std::fs::File;
use std::io::Read;
use hwr_hod_parser::iff::IffChunk;

fn print_kdop(chunk: &IffChunk) {
    if chunk.id == "KDOP" {
        println!("Found KDOP size {}: {:02x?}", chunk.data.len(), chunk.data);
    }
    for child in &chunk.children {
        print_kdop(child);
    }
}

fn main() {
    let file_path = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod";
    let mut file = File::open(file_path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut cursor = std::io::Cursor::new(buffer);
    while cursor.position() < cursor.get_ref().len() as u64 {
        if let Ok(chunk) = IffChunk::read_chunk(&mut cursor) {
            print_kdop(&chunk);
        } else {
            break;
        }
    }
}
