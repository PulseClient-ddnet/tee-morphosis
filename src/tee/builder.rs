//! # Module with builder

use crate::error::Result;
use crate::tee::Tee;
use crate::tee::uv::UV;
use bytes::Bytes;
use image::ImageFormat;

#[derive(Debug, Default, Clone)]
pub struct TeeBuilder {
    data: Option<Bytes>,
    format: Option<ImageFormat>,
    #[cfg(feature = "net")]
    url: Option<String>,
    uv: Option<UV>,
}

impl TeeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_data(
        mut self,
        data: Bytes,
        format: ImageFormat,
    ) -> Self {
        self.data = Some(data);
        self.format = Some(format);
        self
    }

    #[cfg(feature = "net")]
    pub fn with_url(
        mut self,
        url: &str,
    ) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn with_uv(
        mut self,
        uv: UV,
    ) -> Self {
        self.uv = Some(uv);
        self
    }

    #[cfg(feature = "net")]
    pub async fn build(self) -> Result<Tee> {
        use crate::error::TeeError;

        match (self.data, self.format, self.url, self.uv) {
            (Some(data), Some(format), _, uv) => match uv {
                Some(uv) => Tee::new_with_uv(data, uv, format),
                None => Tee::new(data, format),
            },
            (None, None, Some(url), uv) => match uv {
                Some(uv) => Tee::new_from_url_with_uv(&url, uv).await,
                None => Tee::new_from_url(&url).await,
            },
            _ => Err(TeeError::InvalidBuilderConfiguration),
        }
    }

    #[cfg(not(feature = "net"))]
    pub fn build(self) -> Result<Tee> {
        use crate::tee::TeeError;
        match (self.data, self.format, self.uv) {
            (Some(data), Some(format), uv) => match uv {
                Some(uv) => Tee::new_with_uv(data, uv, format),
                None => Tee::new(data, format),
            },
            _ => Err(TeeError::InvalidBuilderConfiguration),
        }
    }
}
