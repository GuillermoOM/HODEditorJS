import re

with open("parser/src/hod.rs", "r") as f:
    data = f.read()

# Replace LMIP preservation logic
old_logic = """                    // Extract original LMIP and STAT chunks from HVMD for preservation
                    for child in &chunk.children {
                        if child.id == "LMIP" {
                            original_lmip_chunks.push(child.clone());
                        } else if child.id == "STAT" {
                            original_stat_chunks.push(child.clone());
                        }
                    }"""

new_logic = """                    // Extract original LMIP/TEXM and STAT/MATT chunks from HVMD for preservation
                    for child in &chunk.children {
                        if child.id == "LMIP" || child.id == "TEXM" {
                            let mut child_clone = child.clone();
                            child_clone.id = "LMIP".to_string();
                            original_lmip_chunks.push(child_clone);
                        } else if child.id == "STAT" || child.id == "MATT" {
                            let mut child_clone = child.clone();
                            child_clone.id = "STAT".to_string();
                            original_stat_chunks.push(child_clone);
                        }
                    }"""

if old_logic in data:
    data = data.replace(old_logic, new_logic)
    print("Patched LMIP/MATT extraction logic!")
else:
    print("Could not find old logic to patch.")

with open("parser/src/hod.rs", "w") as f:
    f.write(data)
