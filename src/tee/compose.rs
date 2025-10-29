//! # Module with composition options

use image::ImageFormat;

use crate::tee::parts::EyeType;

#[derive(Debug, Clone)]
pub struct ComposeOptions {
    pub eye_type: Option<EyeType>,
    pub format: Option<ImageFormat>,
}

impl Default for ComposeOptions {
    fn default() -> Self {
        Self {
            eye_type: Some(EyeType::Happy),
            format: Some(ImageFormat::Png),
        }
    }
}
