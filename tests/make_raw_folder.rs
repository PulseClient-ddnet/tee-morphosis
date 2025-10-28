#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use bytes::Bytes;
    use tee_morphosis::tee::{EyeType, Tee, uv::TEE_UV_LAYOUT};

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
    /// Тестирует сохранение каждой отдельной части TeeRaw в файлы.
    fn test_save_raw_parts() {
        // 1. Подготовка
        let fixture = fixture_path();
        assert!(
            fixture.exists(),
            "Test fixture not found at {:?}. Please download a skin and place it there.",
            fixture
        );
        let skin_data = fs::read(&fixture).expect("Failed to read fixture file");
        let tee = Tee::new(
            Bytes::from(skin_data),
            TEE_UV_LAYOUT,
            image::ImageFormat::Png,
        )
        .expect("Failed to parse TeeRaw");
        let output_dir = setup_output_dir("raws");

        // 2. Сохранение частей
        // Тело
        tee.body
            .value
            .save(output_dir.join("body.png"))
            .expect("Failed to save body");
        tee.body
            .shadow
            .save(output_dir.join("body_shadow.png"))
            .expect("Failed to save body shadow");

        // Ноги
        tee.feet
            .value
            .save(output_dir.join("feet.png"))
            .expect("Failed to save feet");
        tee.feet
            .shadow
            .save(output_dir.join("feet_shadow.png"))
            .expect("Failed to save feet shadow");

        // Руки
        tee.hand
            .value
            .save(output_dir.join("hand.png"))
            .expect("Failed to save hand");
        tee.hand
            .shadow
            .save(output_dir.join("hand_shadow.png"))
            .expect("Failed to save hand shadow");

        // Глаза (все 6 типов)
        for eye_type in [
            EyeType::Normal,
            EyeType::Angry,
            EyeType::Pain,
            EyeType::Happy,
            EyeType::Empty,
            EyeType::Surprise,
        ] {
            let eyes_image = tee.get_eye(eye_type);
            let filename = format!("eye_{:?}.png", eye_type).to_lowercase();
            eyes_image
                .save(output_dir.join(filename))
                .expect("Failed to save eyes");
        }

        // 3. Проверка
        let files: Vec<_> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        // Ожидаем 2(тело) + 2(ноги) + 2(руки) + 6(глаза) = 12 файлов
        assert_eq!(files.len(), 12);
        println!("✅ Raw parts successfully saved to: {:?}", output_dir);
    }
    #[cfg(feature = "net")]
    #[tokio::test]
    /// Тестирует сохранение каждой отдельной части TeeRaw в файлы.
    async fn test_save_raw_parts_from_url() {
        // 1. Подготовка
        let tee = Tee::new_from_url(
            "https://teedata.net/databasev2/skins/glow_rainbow/glow_rainbow.png",
            TEE_UV_LAYOUT,
        )
        .await
        .unwrap();

        let output_dir = setup_output_dir("net_raws");

        // 2. Сохранение частей
        // Тело
        tee.body
            .value
            .save(output_dir.join("body.png"))
            .expect("Failed to save body");
        tee.body
            .shadow
            .save(output_dir.join("body_shadow.png"))
            .expect("Failed to save body shadow");

        // Ноги
        tee.feet
            .value
            .save(output_dir.join("feet.png"))
            .expect("Failed to save feet");
        tee.feet
            .shadow
            .save(output_dir.join("feet_shadow.png"))
            .expect("Failed to save feet shadow");

        // Руки
        tee.hand
            .value
            .save(output_dir.join("hand.png"))
            .expect("Failed to save hand");
        tee.hand
            .shadow
            .save(output_dir.join("hand_shadow.png"))
            .expect("Failed to save hand shadow");

        // Глаза (все 6 типов)
        for eye_type in [
            EyeType::Normal,
            EyeType::Angry,
            EyeType::Pain,
            EyeType::Happy,
            EyeType::Empty,
            EyeType::Surprise,
        ] {
            let eyes_image = tee.get_eye(eye_type);
            let filename = format!("eye_{:?}.png", eye_type).to_lowercase();
            eyes_image
                .save(output_dir.join(filename))
                .expect("Failed to save eyes");
        }

        // 3. Проверка
        let files: Vec<_> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        // Ожидаем 2(тело) + 2(ноги) + 2(руки) + 6(глаза) = 12 файлов
        assert_eq!(files.len(), 12);
        println!("✅ Raw parts successfully saved to: {:?}", output_dir);
    }
}
