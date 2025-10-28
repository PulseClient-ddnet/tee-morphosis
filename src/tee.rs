//! # Tee Module
//!
//! This module provides the core logic for parsing, managing, and compositing 2D
//! character assets, specifically designed for "Tee" characters.
//!
//! ## Main Components
//!
//! *   **[`Tee`]**: The central struct representing a character. It holds
//!     all the visual components, such as the body, feet, hands, and various eye
//!     states, each separated from its shadow.
//!
//! *   **[`UV`]**: A blueprint that defines the coordinates and dimensions
//!     of each part on a source "parts" image. The `Tee::new` function uses this map
//!     to know where to extract each component from.
//!
//! *   **[`Skin`]**: Represents the base skin image onto which the
//!     `Tee` parts are drawn. It also contains the placement information (`SkinPS`)
//!     for where to draw each part on the canvas.
//!
//! ## Typical Workflow
//!
//! The typical usage involves two main steps: parsing the character parts and then
//! compositing them onto a skin.
//!
//! 1.  **Parsing**: Load a raw image containing all the character parts and a `UV`
//!     map that describes its layout. Use either [`Tee::new`] (from bytes) or
//!     [`Tee::new_from_url`] (from a web URL) to get a parsed `Tee` instance.
//!
//! 2.  **Compositing**: With a `Tee` instance and a `Skin` asset, call the
//!     [`Tee::compose`] method. This method overlays the Tee's parts onto the skin
//!     in the correct order and with the specified eye type, producing the final
//!     character image as a byte vector.
//!
//! ## Example
//!
//! ```rust,ignore
//! use tee_morphosis::tee::{Tee, uv::{UV, TEE_UV_LAYOUT}, skin::{Skin, TEE_SKIN_LAYOUT}, EyeType};
//! use tee_morphosis::error::Result;
//! use image::ImageFormat;
//!
//! let uv_map = TEE_UV_LAYOUT; // Assuming a default implementation
//!
//! // Parse the Tee from a URL.
//! let tee = Tee::new_from_url("https://example.com/tee_parts.png", uv_map).await?;
//!
//! let skin = TEE_SKIN_LAYOUT;
//!
//! // Compose the final image with happy eyes, in PNG format.
//! let final_image_bytes = tee.compose(skin, EyeType::Happy, ImageFormat::Png)?;
//!
//! // `final_image_bytes` now contains the complete character image.
//! ```

pub mod skin;
pub mod uv;

use std::io::Cursor;

use bytes::Bytes;
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, RgbaImage, imageops};
use tracing::{debug, error, info, instrument}; // Added `Instrument`

use crate::{
    error::{Result, TeeError},
    tee::{
        skin::{Skin, SkinPS},
        uv::{Part, UV},
    },
};

/// Represents a parsed Tee character, containing all its visual components.
#[derive(Debug, Clone, PartialEq)]
pub struct Tee {
    pub body: WithShadow,
    pub feet: WithShadow,
    /// An array of eye images, ordered as follows:
    /// [Normal, Angry, Pain, Happy, Empty, Surprise]
    pub eye: [EyeTypeData; 6],
    pub hand: WithShadow,
}

/// A struct holding a part of the Tee and its corresponding shadow.
#[derive(Debug, Clone, PartialEq)]
pub struct WithShadow {
    pub value: RgbaImage,
    pub shadow: RgbaImage,
}

/// An enum representing the different states of the Tee's eyes, each holding its corresponding image.
#[derive(Debug, Clone, PartialEq)]
pub enum EyeTypeData {
    Normal(RgbaImage),
    Angry(RgbaImage),
    Pain(RgbaImage),
    Happy(RgbaImage),
    Empty(RgbaImage),
    Surprise(RgbaImage),
}

/// An enum to specify the desired eye state for the Tee.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EyeType {
    Normal,
    Angry,
    Pain,
    Happy,
    Empty,
    Surprise,
}

impl EyeType {
    /// Returns the array index corresponding to this eye type.
    pub const fn index(&self) -> usize {
        match self {
            EyeType::Normal => 0,
            EyeType::Angry => 1,
            EyeType::Pain => 2,
            EyeType::Happy => 3,
            EyeType::Empty => 4,
            EyeType::Surprise => 5,
        }
    }
}

impl Tee {
    /// Parses a `Tee` struct from raw image data.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes of an image containing the [Tee] parts.
    /// * `uv` - A [UV] struct containing the coordinates and dimensions for each part on the source image.
    /// * `format` - The [ImageFormat] of the input data (e.g., PNG, JPEG).
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Tee)` on successful parsing, or `Err(TeeError)` on failure.
    #[instrument(level = "info", skip(data, uv), fields(format = ?format))]
    pub fn new(
        data: Bytes,
        uv: UV,
        format: ImageFormat,
    ) -> Result<Self> {
        let mut img = ImageReader::new(Cursor::new(data));
        img.set_format(format);
        let img = img.decode()?;
        let img_dimensions = img.dimensions();

        debug!(image_dimensions = ?img_dimensions, "Image decoded successfully.");

        if img_dimensions != uv.container {
            error!(
                expected = ?uv.container,
                found = ?img_dimensions,
                "Invalid image dimensions."
            );
            return Err(TeeError::InvalidDimensions {
                expected: uv.container,
                found: img_dimensions,
            });
        }

        debug!("Extracting body parts.");
        let body = WithShadow {
            value: extract_part(&img, uv.body)?,
            shadow: extract_part(&img, uv.body_shadow)?,
        };

        debug!("Extracting feet parts.");
        let feet = WithShadow {
            value: extract_part(&img, uv.feet)?,
            shadow: extract_part(&img, uv.feet_shadow)?,
        };

        debug!("Extracting hand parts.");
        let hand = WithShadow {
            value: extract_part(&img, uv.hand)?,
            shadow: extract_part(&img, uv.hand_shadow)?,
        };

        debug!("Extracting eye parts.");
        let eye = [
            EyeTypeData::Normal(extract_part(&img, uv.eyes[0])?),
            EyeTypeData::Angry(extract_part(&img, uv.eyes[1])?),
            EyeTypeData::Pain(extract_part(&img, uv.eyes[2])?),
            EyeTypeData::Happy(extract_part(&img, uv.eyes[3])?),
            EyeTypeData::Empty(extract_part(&img, uv.eyes[4])?),
            EyeTypeData::Surprise(extract_part(&img, uv.eyes[5])?),
        ];

        Ok(Self {
            body,
            feet,
            eye,
            hand,
        })
    }

    #[cfg(feature = "net")]
    #[cfg_attr(docsrs, doc(cfg(feature = "net")))]
    /// Asynchronously fetches a [Tee] skin from a URL and parses it.
    /// The image format is determined from the `Content-Type` header of the response.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice that holds the URL of the skin image.
    /// * `uv` - A [UV] struct containing the coordinates and dimensions for each part.
    ///
    /// # Returns
    ///
    /// A [Result] which is Ok([Tee]) on successful fetching and parsing, or Err([TeeError]) on failure.
    #[instrument(level = "info", skip(uv), fields(url = %url))]
    pub async fn new_from_url(
        url: &str,
        uv: UV,
    ) -> Result<Self> {
        let response = reqwest::get(url).await.map_err(|e| {
            error!(error = %e, "Failed to send request.");
            TeeError::Reqwest(e)
        })?;

        // Determine format from Content-Type header
        let format = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|mime| ImageFormat::from_mime_type(mime))
            .ok_or_else(|| {
                error!("'Content-Type' header is missing or invalid.");
                TeeError::ReqWithOutContentType(url.to_string())
            })?;

        info!(determined_format = ?format, "Image format determined from response header.");

        let bytes = response.bytes().await.map_err(|e| {
            error!(error = %e, "Failed to read bytes from response.");
            TeeError::Reqwest(e)
        })?;

        info!(bytes_len = bytes.len(), "Successfully fetched image data.");

        // Use `instrument` to create a new span for the `Tee::new` call
        Self::new(bytes, uv, format)
    }

    /// Retrieves the image for a specific eye type.
    ///
    /// # Arguments
    ///
    /// * `type` - The `EyeType` to retrieve.
    ///
    /// # Returns
    ///
    /// A reference to the `RgbaImage` corresponding to the requested eye type.
    /// This function will panic if the internal state is inconsistent, which is
    /// prevented by the construction logic in `Tee::new`.
    #[instrument(level = "debug", skip(self), fields(eye_type = ?r#type))]
    pub fn get_eye(
        &self,
        r#type: EyeType,
    ) -> &RgbaImage {
        let index = r#type.index();
        match (&r#type, &self.eye[index]) {
            (EyeType::Normal, EyeTypeData::Normal(img)) => img,
            (EyeType::Angry, EyeTypeData::Angry(img)) => img,
            (EyeType::Pain, EyeTypeData::Pain(img)) => img,
            (EyeType::Happy, EyeTypeData::Happy(img)) => img,
            (EyeType::Empty, EyeTypeData::Empty(img)) => img,
            (EyeType::Surprise, EyeTypeData::Surprise(img)) => img,

            // This is a safety check that should never be hit if the Tee is constructed correctly.
            _ => unreachable!(
                "Invariant violation: eye type at index {} does not match the requested type.",
                index
            ),
        }
    }

    /// Composites the Tee parts onto a base skin image to create a final character portrait.
    ///
    /// # Arguments
    ///
    /// * `skin` - The base `Skin` to draw the Tee parts onto.
    /// * `eye_type` - The `EyeType` to use for the eyes in the final image.
    /// * `img_format` - The desired `ImageFormat` for the output bytes (e.g., PNG, JPEG).
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Bytes)` containing the final image data on success,
    /// or `Err(TeeError)` on failure.
    #[instrument(level = "info", skip(self, skin), fields(eye_type = ?eye_type, img_format = ?img_format, skin_container = ?skin.container))]
    pub fn compose(
        &self,
        skin: Skin,
        eye_type: EyeType,
        img_format: ImageFormat,
    ) -> Result<Bytes> {
        let mut canvas = RgbaImage::new(skin.container.0, skin.container.1);
        let mut compose = |layer: &RgbaImage, ((x, y), (w, h)): SkinPS| {
            debug!(
                "Composing layer at position ({}, {}) with size ({}, {})",
                x, y, w, h
            );
            imageops::overlay(
                &mut canvas,
                &imageops::resize(layer, w, h, imageops::FilterType::Triangle),
                x,
                y,
            );
        };

        // Layering order is important for correct appearance
        compose(&self.feet.shadow, skin.back_feet); // back feet shadow
        compose(&self.body.shadow, skin.body); // body shadow
        compose(&self.feet.shadow, skin.front_feet); // front feet shadow
        compose(&self.feet.value, skin.back_feet); // back feet
        compose(&self.body.value, skin.body); // body
        compose(&self.feet.value, skin.front_feet); // front feet
        let eye = self.get_eye(eye_type);
        compose(&eye, skin.first_eyes); // first eye
        compose(&imageops::flip_horizontal(eye), skin.second_eyes); // second eye (flipped)

        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        debug!(
            "Writing composed image to buffer in format: {:?}",
            img_format
        );
        canvas.write_to(&mut cursor, img_format)?;

        info!(output_size = buf.len(), "Successfully composed Tee image.");
        Ok(Bytes::from(buf))
    }
}

/// Extracts a rectangular part from a source image.
///
/// # Arguments
///
/// * `img` - A reference to the source `DynamicImage`.
/// * `part` - A `Part` struct defining the coordinates (`x`, `y`) and dimensions (`w`, `h`) of the area to extract.
///
/// # Returns
///
/// A `Result` which is `Ok(RgbaImage)` containing the extracted part, or `Err(TeeError::OutOfBounds)` if the part's dimensions fall outside the source image.
///
/// # Errors
///
/// Returns `TeeError::OutOfBounds` if the dimensions provided fall out of bounds.
#[instrument(level = "debug", skip(img), fields(part = ?part))]
fn extract_part(
    img: &DynamicImage,
    part: Part,
) -> Result<RgbaImage> {
    let (img_width, img_height) = img.dimensions();

    if part.x + part.w > img_width || part.y + part.h > img_height {
        error!(
            image_width = img_width,
            image_height = img_height,
            "Failed to extract part: out of bounds."
        );
        return Err(TeeError::OutOfBounds {
            part,
            width: img_width,
            height: img_height,
        });
    }

    let cropped_image = img.view(part.x, part.y, part.w, part.h).to_image();
    Ok(cropped_image)
}
