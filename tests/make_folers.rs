#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use bytes::Bytes;
    use image::EncodableLayout;
    use tee_morphosis::tee::{Tee, parts::EyeType, skin::TEE_SKIN_LAYOUT, uv::TEE_UV_LAYOUT};

    fn fixture_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(".ref");
        path.push("test_skin.png");
        path
    }

    fn setup_output_dir(dir_name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(".test");
        path.push(dir_name);
        if path.exists() {
            fs::remove_dir_all(&path).expect("Failed to clean up output directory");
        }
        fs::create_dir_all(&path).expect("Failed to create output directory");
        path
    }

    #[test]
    fn test_save_raw_parts() {
        let fixture = fixture_path();
        assert!(
            fixture.exists(),
            "Test fixture not found at {:?}. Please download a skin and place it there.",
            fixture
        );
        let skin_data = fs::read(&fixture).expect("Failed to read fixture file");
        let tee = Tee::new(Bytes::from(skin_data), image::ImageFormat::Png)
            .expect("Failed to parse TeeRaw");
        let output_dir = setup_output_dir("raws");

        tee.body
            .value
            .save(output_dir.join("body.png"))
            .expect("Failed to save body");
        tee.body
            .shadow
            .save(output_dir.join("body_shadow.png"))
            .expect("Failed to save body shadow");

        tee.feet
            .value
            .save(output_dir.join("feet.png"))
            .expect("Failed to save feet");
        tee.feet
            .shadow
            .save(output_dir.join("feet_shadow.png"))
            .expect("Failed to save feet shadow");

        tee.hand
            .value
            .save(output_dir.join("hand.png"))
            .expect("Failed to save hand");
        tee.hand
            .shadow
            .save(output_dir.join("hand_shadow.png"))
            .expect("Failed to save hand shadow");

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

        let files: Vec<_> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        // Expect 2(body) + 2(feet) + 2(hand) + 6(eye) = 12 files
        assert_eq!(files.len(), 12);
        println!("✅ Raw parts successfully saved to: {:?}", output_dir);
    }

    #[cfg(feature = "net")]
    #[tokio::test]
    async fn test_save_raw_parts_from_url() {
        let tee = Tee::new_from_url_with_uv(
            "https://teedata.net/databasev2/skins/glow_rainbow/glow_rainbow.png",
            TEE_UV_LAYOUT,
        )
        .await
        .unwrap();

        let output_dir = setup_output_dir("net_raws");

        tee.body
            .value
            .save(output_dir.join("body.png"))
            .expect("Failed to save body");
        tee.body
            .shadow
            .save(output_dir.join("body_shadow.png"))
            .expect("Failed to save body shadow");

        tee.feet
            .value
            .save(output_dir.join("feet.png"))
            .expect("Failed to save feet");
        tee.feet
            .shadow
            .save(output_dir.join("feet_shadow.png"))
            .expect("Failed to save feet shadow");

        tee.hand
            .value
            .save(output_dir.join("hand.png"))
            .expect("Failed to save hand");
        tee.hand
            .shadow
            .save(output_dir.join("hand_shadow.png"))
            .expect("Failed to save hand shadow");

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

        let files: Vec<_> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(files.len(), 12);
        println!("✅ Raw parts successfully saved to: {:?}", output_dir);
    }

    #[test]
    fn test_save_composed_image() {
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
            let filename = format!("composed_{:?}.webp", eye_type).to_lowercase();
            fs::write(output_dir.join(filename), image_bytes.as_bytes())
                .expect("Failed to write composed image");
        }

        let files: Vec<_> = fs::read_dir(&output_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(files.len(), 6);
        println!("✅ Composed images successfully saved to: {:?}", output_dir);
    }
}
