use roxmltree::Document;
fn main() {
    let xml = std::fs::read_to_string("../testing/ter_centaur/ter_centaur.DAE").unwrap();
    let doc = Document::parse(&xml).unwrap();
    if let Some(node) = doc.descendants().find(|n| n.attribute("name") == Some("JNT[EngineNozzle1]")) {
        println!("Found EngineNozzle1");
        for child in node.children() {
            println!("  Child: {:?}, tag: {:?}", child, child.tag_name().name());
            if child.has_tag_name("translate") {
                println!("    translate text: {:?}", child.text());
            }
        }
    }
}
