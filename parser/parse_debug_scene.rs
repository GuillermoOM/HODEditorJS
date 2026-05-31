use roxmltree::Document;
fn main() {
    let xml = std::fs::read_to_string("../testing/ter_centaur/ter_centaur.DAE").unwrap();
    let doc = Document::parse(&xml).unwrap();
    if let Some(scene) = doc.descendants().find(|n| n.has_tag_name("visual_scene")) {
        for child in scene.children().filter(|n| n.has_tag_name("node")) {
            println!("Node: {:?}", child.attribute("name"));
        }
    }
}
