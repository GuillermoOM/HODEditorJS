fn main() {
    let mut transform = [[0.0_f32; 4]; 4];
    transform[0][0] = 1.0; transform[1][1] = 1.0; transform[2][2] = 1.0; transform[3][3] = 1.0;
    
    transform[3][0] = 0.0;
    transform[3][1] = -1.35;
    transform[3][2] = -12.0;

    let rad = 180.0_f32 * std::f32::consts::PI / 180.0;
    let (s, c) = (rad.sin(), rad.cos());
    let t = 1.0 - c;
    let ax = 0.0_f32;
    let ay = 1.0_f32;
    let az = 0.0_f32;

    let rot = [
        [t*ax*ax + c,     t*ax*ay - s*az, t*ax*az + s*ay, 0.0],
        [t*ax*ay + s*az,  t*ay*ay + c,    t*ay*az - s*ax, 0.0],
        [t*ax*az - s*ay,  t*ay*az + s*ax, t*az*az + c,    0.0],
        [0.0,             0.0,             0.0,            1.0],
    ];

    let mut result = [[0.0_f32; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            result[r][c] = rot[r][0] * transform[0][c]
                + rot[r][1] * transform[1][c]
                + rot[r][2] * transform[2][c]
                + rot[r][3] * transform[3][c];
        }
    }
    
    println!("Result: {:?}", result);
}
