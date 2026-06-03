import re

filepath = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/parser/src/hod.rs"

with open(filepath, 'r') as f:
    content = f.read()

dxt1_start = content.find("fn compress_dxt1(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {")
dxt1_end = content.find("}", content.find("out\n", dxt1_start)) + 1
if dxt1_start != -1 and dxt1_end != -1:
    new_dxt1 = """fn compress_dxt1(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let out_size = ((width.max(1) + 3) / 4) * ((height.max(1) + 3) / 4) * 8;
    let mut out = vec![0u8; out_size];
    texpresso::Format::Bc1.compress(rgba, width, height, texpresso::Params::default(), &mut out);
    out
}"""
    content = content[:dxt1_start] + new_dxt1 + content[dxt1_end:]

dxt3_start = content.find("fn compress_dxt3(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {")
dxt3_end = content.find("}", content.find("out\n", dxt3_start)) + 1
if dxt3_start != -1 and dxt3_end != -1:
    new_dxt3 = """fn compress_dxt3(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let out_size = ((width.max(1) + 3) / 4) * ((height.max(1) + 3) / 4) * 16;
    let mut out = vec![0u8; out_size];
    texpresso::Format::Bc2.compress(rgba, width, height, texpresso::Params::default(), &mut out);
    out
}"""
    content = content[:dxt3_start] + new_dxt3 + content[dxt3_end:]

dxt5_start = content.find("fn compress_dxt5(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {")
dxt5_end = content.find("}", content.find("out\n", dxt5_start)) + 1
if dxt5_start != -1 and dxt5_end != -1:
    new_dxt5 = """fn compress_dxt5(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let out_size = ((width.max(1) + 3) / 4) * ((height.max(1) + 3) / 4) * 16;
    let mut out = vec![0u8; out_size];
    texpresso::Format::Bc3.compress(rgba, width, height, texpresso::Params::default(), &mut out);
    out
}"""
    content = content[:dxt5_start] + new_dxt5 + content[dxt5_end:]


with open(filepath, 'w') as f:
    f.write(content)
