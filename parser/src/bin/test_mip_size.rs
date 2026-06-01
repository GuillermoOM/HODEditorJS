fn calc_total_size(mut w: usize, mut h: usize, format: &str) -> usize {
    let mut total = 0;
    while w >= 1 || h >= 1 {
        let mip_w = std::cmp::max(1, w);
        let mip_h = std::cmp::max(1, h);
        if format == "DXT1" {
            total += std::cmp::max(8, (mip_w * mip_h) / 2);
        } else if format == "DXT5" {
            total += std::cmp::max(16, mip_w * mip_h);
        }
        if w == 1 && h == 1 { break; }
        w /= 2;
        h /= 2;
    }
    total
}

fn main() {
    println!("DXT1 1024x1024 total size: {}", calc_total_size(1024, 1024, "DXT1"));
    println!("DXT1 1024x1024 top mip: {}", 1024 * 1024 / 2);
}
