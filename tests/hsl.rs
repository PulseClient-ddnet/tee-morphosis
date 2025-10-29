#[cfg(test)]
mod tests {

    /// Константа из DDNet для lighting
    const DARKEST_LGT: f32 = 0.5;

    /// Распаковывает DDNet цвет (упакованный HSL) в компоненты HSL
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

    /// Упаковывает HSL (0-1 диапазон) в DDNet формат u32
    /// l_value должен быть уже "распакованным" (0-1)
    pub fn hsl_to_ddnet_color(
        h: f32,
        s: f32,
        l: f32,
    ) -> u32 {
        // Обратная операция UnclampLighting
        let l_compressed = (l - DARKEST_LGT) / (1.0 - DARKEST_LGT);
        let l_clamped = l_compressed.clamp(0.0, 1.0);

        let h_byte = (h * 255.0).clamp(0.0, 255.0) as u32;
        let s_byte = (s * 255.0).clamp(0.0, 255.0) as u32;
        let l_byte = (l_clamped * 255.0).clamp(0.0, 255.0) as u32;

        (h_byte << 16) | (s_byte << 8) | l_byte + 1
    }

    /// Конвертирует HSL в RGB
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

    #[test]
    fn test_ddnet_color_1900500() {
        let color = 1900500;

        println!("Raw color: 0x{:06X} ({})", color, color);

        let (h, s, l) = ddnet_color_to_hsl(color);
        println!(
            "Decoded HSL: H={:.0}°, S={:.0}%, L={:.0}%",
            h * 360.0,
            s * 100.0,
            l * 100.0
        );

        // Ожидается: H: 40°, S: 100%, L: 92%
        assert!((h * 360.0 - 40.0).abs() < 1.0, "Hue should be ~40°");
        assert!(
            (s * 100.0 - 100.0).abs() < 1.0,
            "Saturation should be ~100%"
        );
        assert!((l * 100.0 - 92.0).abs() < 1.0, "Lightness should be ~92%");

        let (r, g, b) = hsl_to_rgb(h, s, l);
        println!(
            "Converted to RGB: R={}, G={}, B={} (#{:02X}{:02X}{:02X})",
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8
        );

        assert!((r * 255.0 - 255.0).abs() < 2.0, "R should be ~255");
        assert!((g * 255.0 - 240.0).abs() < 2.0, "G should be ~240");
        assert!((b * 255.0 - 212.0).abs() < 2.0, "B should be ~212");
    }

    #[test]
    fn test_roundtrip() {
        let original_color = 1900500;
        let (h, s, l) = ddnet_color_to_hsl(original_color);
        let packed_back = hsl_to_ddnet_color(h, s, l);

        println!(
            "Original: 0x{:06X}, Roundtrip: 0x{:06X}",
            original_color, packed_back
        );
        assert_eq!(
            original_color, packed_back,
            "Color should survive roundtrip"
        );
    }
}
