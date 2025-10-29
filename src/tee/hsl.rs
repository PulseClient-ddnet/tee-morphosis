use image::RgbaImage;

const DARKEST_LGT: f32 = 0.5;

pub fn ddnet_color_to_hsl(color: u32) -> (f32, f32, f32) {
    let h_raw = ((color >> 16) & 0xFF) as f32;
    let s_raw = ((color >> 8) & 0xFF) as f32;
    let l_raw = (color & 0xFF) as f32;

    let h = h_raw / 255.0;
    let s = s_raw / 255.0;
    let l_compressed = l_raw / 255.0;

    // Применяем UnclampLighting
    let l = DARKEST_LGT + l_compressed * (1.0 - DARKEST_LGT);

    (h, s, l)
}

fn hsl_to_rgb(
    h: f32,
    s: f32,
    l: f32,
) -> (f32, f32, f32) {
    let h1 = h * 6.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h1 % 2.0) - 1.0).abs());

    let (r, g, b) = match h1.floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 | 6 => (c, 0.0, x),
        _ => (c, 0.0, x),
    };

    let m = l - (c / 2.0);
    (r + m, g + m, b + m)
}

pub fn img_hsl_transform(
    img: &mut RgbaImage,
    (h, s, l): (f32, f32, f32),
) {
    let (col_r, col_g, col_b) = hsl_to_rgb(h, s, l);

    for pixel in img.pixels_mut() {
        pixel[0] = ((pixel[0] as f32 / 255.0 * col_r) * 255.0).clamp(0.0, 255.0) as u8;
        pixel[1] = ((pixel[1] as f32 / 255.0 * col_g) * 255.0).clamp(0.0, 255.0) as u8;
        pixel[2] = ((pixel[2] as f32 / 255.0 * col_b) * 255.0).clamp(0.0, 255.0) as u8;
    }
}
