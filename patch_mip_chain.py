import re

filepath = "/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/mod-tools/HODEditorJS/parser/src/hod.rs"

with open(filepath, 'r') as f:
    content = f.read()

old_func = """fn generate_mip_chain(
    rgba: &[u8],
    width: usize,
    height: usize,
    max_mips: usize,
) -> Vec<(Vec<u8>, usize, usize)> {
    let mut mips = Vec::new();
    let mut cur_rgba = rgba.to_vec();
    let mut cur_w = width;
    let mut cur_h = height;

    for _ in 0..max_mips {
        mips.push((cur_rgba.clone(), cur_w, cur_h));
        if cur_w <= 1 && cur_h <= 1 {
            break;
        }
        let next_w = std::cmp::max(1, cur_w / 2);
        let next_h = std::cmp::max(1, cur_h / 2);
        let mut next = vec![0u8; next_w * next_h * 4];
        for ny in 0..next_h {
            for nx in 0..next_w {
                let sx = nx * 2;
                let sy = ny * 2;
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut a = 0u32;
                let mut cnt = 0u32;
                for dy in 0..2 {
                    for dx in 0..2 {
                        let px = sx + dx;
                        let py = sy + dy;
                        if px < cur_w && py < cur_h {
                            let i = (py * cur_w + px) * 4;
                            r += cur_rgba[i] as u32;
                            g += cur_rgba[i + 1] as u32;
                            b += cur_rgba[i + 2] as u32;
                            a += cur_rgba[i + 3] as u32;
                            cnt += 1;
                        }
                    }
                }
                let ni = (ny * next_w + nx) * 4;
                next[ni] = (r / cnt) as u8;
                next[ni + 1] = (g / cnt) as u8;
                next[ni + 2] = (b / cnt) as u8;
                next[ni + 3] = (a / cnt) as u8;
            }
        }
        cur_rgba = next;
        cur_w = next_w;
        cur_h = next_h;
    }
    mips
}"""

new_func = """fn generate_mip_chain(
    rgba: &[u8],
    width: usize,
    height: usize,
    max_mips: usize,
) -> Vec<(Vec<u8>, usize, usize)> {
    let mut mips = Vec::new();
    let mut cur_rgba = rgba.to_vec();
    let mut cur_w = width;
    let mut cur_h = height;

    for _ in 0..max_mips {
        mips.push((cur_rgba.clone(), cur_w, cur_h));
        if cur_w <= 1 && cur_h <= 1 {
            break;
        }
        let next_w = std::cmp::max(1, cur_w / 2);
        let next_h = std::cmp::max(1, cur_h / 2);
        
        if let Some(img) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(cur_w as u32, cur_h as u32, cur_rgba.clone()) {
            let resized = image::imageops::resize(
                &img,
                next_w as u32,
                next_h as u32,
                image::imageops::FilterType::Lanczos3,
            );
            cur_rgba = resized.into_raw();
        }
        
        cur_w = next_w;
        cur_h = next_h;
    }
    mips
}"""

content = content.replace(old_func, new_func)

with open(filepath, 'w') as f:
    f.write(content)
