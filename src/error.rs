//! # Error Module

pub type Result<T> = std::result::Result<T, TeeError>;

#[derive(thiserror::Error, Debug)]
pub enum TeeError {
    #[cfg(feature = "net")]
    #[error("Got error then work with url context: {0}")]
    Reqwest(reqwest::Error),
    #[cfg(feature = "net")]
    #[error("Got error then using task in async context: {0}")]
    Join(tokio::task::JoinError),
    #[cfg(feature = "net")]
    #[error("Req does not contains any img content type: {0}")]
    ReqWithOutContentType(String),

    // Добавить в src/error.rs
    #[error("Invalid builder configuration. Provide either data+format or url")]
    InvalidBuilderConfiguration,

    #[error("Got error then work with image: {0}")]
    Image(#[from] image::ImageError),

    #[error(
        "The requested part {part:?} is outside the image bounds (width: {width}, height: {height})"
    )]
    OutOfBounds {
        part: crate::tee::uv::UVPart,
        width: u32,
        height: u32,
    },

    #[error("Invalid image dimensions. Expected {expected:?}, but found {found:?}.")]
    InvalidDimensions {
        expected: (u32, u32),
        found: (u32, u32),
    },
}
