#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use bytes::Bytes;
    use image::EncodableLayout;
    use tee_morphosis::tee::{EyeType, Tee, skin::TEE_SKIN_LAYOUT};

    /// Возвращает путь к тестовому файлу скина.
    fn fixture_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(".ref");
        path.push("test_skin.png");
        path
    }

    /// Очищает и создает выходную директорию для тестов.
    fn setup_output_dir(dir_name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(".test");
        path.push(dir_name);
        // Очищаем директорию перед каждым тестом для чистоты эксперимента
        if path.exists() {
            fs::remove_dir_all(&path).expect("Failed to clean up output directory");
        }
        fs::create_dir_all(&path).expect("Failed to create output directory");
        path
    }

    #[test]
    /// Тестирует создание и сохранение итогового скомпонованного изображения.
    fn test_save_composed_image() {
        // 1. Подготовка
        let fixture = fixture_path();
        assert!(
            fixture.exists(),
            "Test fixture not found at {:?}. Please download a skin and place it there.",
            fixture
        );
        let skin_data = fs::read(&fixture).expect("Failed to read fixture file");
        let tee = Tee::new(Bytes::from(skin_data), image::ImageFormat::Png)
            .expect("Failed to parse TeeRaw");
        let output_dir = setup_output_dir("composed");

        // 2. Композиция и сохранение
        for eye_type in [
            EyeType::Normal,
            EyeType::Angry,
            EyeType::Pain,
            EyeType::Happy,
            EyeType::Empty,
            EyeType::Surprise,
        ] {
            let image_bytes = tee
                .compose(TEE_SKIN_LAYOUT, eye_type.clone(), image::ImageFormat::WebP)
                .expect("Failed to compose image");
            let filename = format!("composed_{:?}.png", eye_type).to_lowercase();
            fs::write(output_dir.join(filename), image_bytes.as_bytes())
                .expect("Failed to write composed image");
        }

        // 3. Проверка
        let files: Vec<_> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        // Ожидаем 6 файлов (по одному на каждый тип глаз)
        assert_eq!(files.len(), 6);
        println!("✅ Composed images successfully saved to: {:?}", output_dir);
    }
}
