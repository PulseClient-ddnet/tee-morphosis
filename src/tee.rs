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
//! use tee_morphosis::tee::{Tee, uv::{UV, TEE_UV_LAYOUT}, skin::{Skin, TEE_SKIN_LAYOUT}, parts::EyeType};
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

pub mod builder;
pub mod hsl;
pub mod parts;
pub mod skin;
pub mod uv;

use std::{collections::HashMap, io::Cursor};

use bytes::Bytes;
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, RgbaImage, imageops};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    error::{Result, TeeError},
    tee::{
        hsl::{HSL, img_hsl_transform},
        parts::{EyeType, EyeTypeData, TeePart, WithShadow},
        skin::{Skin, SkinPS},
        uv::{TEE_UV_LAYOUT, UV, UVPart},
    },
};

/// Represents a parsed Tee character, containing all its visual components.
///
/// The Tee struct holds all the necessary parts to render a character, including
/// body parts, feet, hands, and various eye states. Each part is stored separately
/// from its shadow to allow for independent manipulation.
#[derive(Debug, Clone, PartialEq)]
pub struct Tee {
    /// The body part of the character, including both the main body and its shadow
    pub body: WithShadow,
    /// The feet parts of the character, including both the main feet and their shadow
    pub feet: WithShadow,
    /// An array of eye images, ordered as follows:
    ///
    /// [Normal, Angry, Pain, Happy, Empty, Surprise]
    pub eye: [EyeTypeData; 6],
    /// The hand parts of the character, including both the main hand and its shadow
    pub hand: WithShadow,
    /// The UV mapping used to extract parts from the source image
    pub used_uv: UV,
}

impl Tee {
    /// Parses a `Tee` struct from raw image data with default [uv]::[TEE_UV_LAYOUT].
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes of an image containing the [Tee] parts.
    /// * `format` - The [ImageFormat] of the input data (e.g., PNG, JPEG).
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Tee)` on successful parsing, or `Err(TeeError)` on failure.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::Tee;
    /// use image::ImageFormat;
    ///
    /// let image_data = std::fs::read("tee_parts.png")?;
    /// let tee = Tee::new(image_data.into(), ImageFormat::Png)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[instrument(level = "info", skip(data), fields(format = ?format))]
    pub fn new(
        data: Bytes,
        format: ImageFormat,
    ) -> Result<Self> {
        Self::new_with_uv(data, TEE_UV_LAYOUT, format)
    }

    /// Parses a `Tee` struct from raw image data with a custom UV layout.
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
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::{Tee, uv::UV};
    /// use image::ImageFormat;
    ///
    /// let image_data = std::fs::read("custom_tee_parts.png")?;
    /// let custom_uv = UV { /* custom layout */ };
    /// let tee = Tee::new_with_uv(image_data.into(), custom_uv, ImageFormat::Png)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[instrument(level = "info", skip(data, uv), fields(format = ?format))]
    pub fn new_with_uv(
        data: Bytes,
        uv: UV,
        format: ImageFormat,
    ) -> Result<Self> {
        trace!("Starting to decode image with format: {:?}", format);
        let img = decode_image(data, format)?;
        let img_dimensions = img.dimensions();

        debug!(image_dimensions = ?img_dimensions, "Image decoded successfully.");

        validate_image_dimensions(img_dimensions, uv.container)?;

        debug!("Extracting all parts from the image.");
        let body = extract_with_shadow(&img, uv.body, uv.body_shadow)?;
        let feet = extract_with_shadow(&img, uv.feet, uv.feet_shadow)?;
        let hand = extract_with_shadow(&img, uv.hand, uv.hand_shadow)?;
        let eye = extract_all_eyes(&img, &uv.eyes)?;

        info!("Successfully parsed all Tee parts from the image.");
        Ok(Self {
            body,
            feet,
            eye,
            hand,
            used_uv: uv,
        })
    }

    #[cfg(feature = "net")]
    #[cfg_attr(docsrs, doc(cfg(feature = "net")))]
    /// Asynchronously fetches a [Tee] skin from a URL and parses it with a custom UV layout.
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
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::{Tee, uv::UV};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let custom_uv = UV { /* custom layout */ };
    ///     let tee = Tee::new_from_url_with_uv("https://example.com/tee.png", custom_uv).await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(level = "info", skip(uv), fields(url = %url))]
    pub async fn new_from_url_with_uv(
        url: &str,
        uv: UV,
    ) -> Result<Self> {
        trace!("Fetching image from URL: {}", url);
        let (bytes, format) = fetch_image_from_url(url).await?;

        info!(
            "Successfully fetched image data, size: {} bytes",
            bytes.len()
        );

        // Use `instrument` to create a new span for the `Tee::new_with_uv` call
        tokio::task::spawn_blocking(move || Self::new_with_uv(bytes, uv, format))
            .await
            .map_err(TeeError::Join)?
    }

    #[cfg(feature = "net")]
    #[cfg_attr(docsrs, doc(cfg(feature = "net")))]
    /// Asynchronously fetches a [Tee] skin from a URL and parses it with the default UV layout.
    /// The image format is determined from the `Content-Type` header of the response.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice that holds the URL of the skin image.
    ///
    /// # Returns
    ///
    /// A [Result] which is Ok([Tee]) on successful fetching and parsing, or Err([TeeError]) on failure.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::Tee;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tee = Tee::new_from_url("https://example.com/tee.png").await?;
    ///     Ok(())
    /// }
    /// ```
    #[instrument(level = "info", fields(url = %url))]
    pub async fn new_from_url(url: &str) -> Result<Self> {
        trace!("Fetching image from URL: {}", url);
        let (bytes, format) = fetch_image_from_url(url).await?;

        info!(
            "Successfully fetched image data, size: {} bytes",
            bytes.len()
        );

        tokio::task::spawn_blocking(move || Self::new(bytes, format))
            .await
            .map_err(TeeError::Join)?
    }

    /// Applies HSL color transformation to specific parts of the Tee.
    ///
    /// # Arguments
    ///
    /// * `hls` - A tuple of (hue, saturation, value) values, each in the range [0.0, 1.0].
    /// * `parts` - A slice of `TeePart` specifying which parts to apply the transformation to.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::{Tee, parts::TeePart};
    /// use tee_morphosis::tee:hsl::ddnet_color_to_hsl;
    ///
    /// let mut tee = Tee::new(/* ... */)?;
    ///
    /// let hsl = ddnet_color_to_hsl(1900500);
    /// tee.apply_hsl_to_parts(hsl, &[TeePart::Body]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[instrument(level = "debug", skip(self), fields(hsl = ?hsl, parts_count = parts.len()))]
    pub fn apply_hsl_to_parts(
        &mut self,
        hsl: (f32, f32, f32),
        parts: &[TeePart],
    ) {
        trace!("Applying HSL transformation to {} parts", parts.len());
        for part in parts {
            match part {
                TeePart::Body => {
                    img_hsl_transform(&mut self.body.value, hsl);
                }
                TeePart::BodyShadow => {
                    img_hsl_transform(&mut self.body.shadow, hsl);
                }
                TeePart::Feet => {
                    img_hsl_transform(&mut self.feet.value, hsl);
                }
                TeePart::FeetShadow => {
                    img_hsl_transform(&mut self.feet.shadow, hsl);
                }
                TeePart::Hand => {
                    img_hsl_transform(&mut self.hand.value, hsl);
                }
                TeePart::HandShadow => {
                    img_hsl_transform(&mut self.hand.shadow, hsl);
                }
            }
        }
        debug!("Successfully applied HSL transformation to specified parts");
    }

    /// Applies HSL color transformation to all parts of the Tee.
    ///
    /// # Arguments
    ///
    /// * `hls` - A tuple of (hue, saturation, value) values, each in the range [0.0, 1.0].
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::Tee;
    /// use tee_morphosis::tee:hsl::ddnet_color_to_hsl;
    ///
    /// let mut tee = Tee::new(/* ... */)?;
    ///
    /// let hsl = ddnet_color_to_hsl(1900500);
    /// tee.apply_hsl_to_all(hsl);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[instrument(level = "debug", skip(self), fields(hls = ?hls))]
    pub fn apply_hsl_to_all(
        &mut self,
        hls: HSL,
    ) {
        trace!("Applying HSL transformation to all parts");
        self.apply_hsl_to_parts(
            hls,
            &[
                TeePart::Body,
                TeePart::BodyShadow,
                TeePart::Feet,
                TeePart::FeetShadow,
                TeePart::Hand,
                TeePart::HandShadow,
            ],
        );
        debug!("Successfully applied HSL transformation to all parts");
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
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::{Tee, parts::EyeType, skin::TEE_SKIN_LAYOUT};
    /// use image::ImageFormat;
    ///
    /// let tee = Tee::new(/* ... */)?;
    /// let result = tee.compose(TEE_SKIN_LAYOUT, EyeType::Happy, ImageFormat::Png)?;
    /// std::fs::write("output.png", result)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[instrument(level = "info", skip(self, skin), fields(eye_type = ?eye_type, img_format = ?img_format, skin_container = ?skin.container))]
    pub fn compose(
        &self,
        skin: Skin,
        eye_type: EyeType,
        img_format: ImageFormat,
    ) -> Result<Bytes> {
        trace!("Starting composition process");
        let mut canvas = RgbaImage::new(skin.container.0, skin.container.1);

        // Define the composition function
        let mut compose = |layer: &RgbaImage, ((x, y), scale): SkinPS, uv_part: UVPart| {
            debug!(
                "Composing layer at position ({}, {}) with size ({}, {}) and scale {}",
                x, y, uv_part.w, uv_part.h, scale
            );
            let (w, h) = skin::scale((uv_part.w, uv_part.h), scale);
            imageops::overlay(
                &mut canvas,
                &imageops::resize(layer, w, h, imageops::FilterType::Triangle),
                x,
                y,
            );
        };

        // Layering order is important for correct appearance
        self.compose_layers(&mut compose, &skin, eye_type);

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

    /// Composites the Tee with PNG format.
    ///
    /// # Arguments
    ///
    /// * `skin` - The base `Skin` to draw the Tee parts onto.
    /// * `eye_type` - The type of eyes to use.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Bytes)` containing the final image data on success,
    /// or `Err(TeeError)` on failure.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::{Tee, skin::TEE_SKIN_LAYOUT};
    ///
    /// let tee = Tee::new(/* ... */)?;
    /// let result = tee.compose_png(TEE_SKIN_LAYOUT, EyeType::Happy)?;
    /// std::fs::write("output.png", result)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[instrument(level = "info", skip(self, skin), fields(skin_container = ?skin.container))]
    pub fn compose_png(
        &self,
        skin: Skin,
        eye_type: EyeType,
    ) -> Result<Bytes> {
        trace!("Composing with default options (happy eyes, PNG format)");
        self.compose(skin, eye_type, ImageFormat::Png)
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
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::{Tee, parts::EyeType};
    ///
    /// let tee = Tee::new(/* ... */)?;
    /// let happy_eye = tee.get_eye(EyeType::Happy);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

    /// Returns all parts of the Tee as a HashMap.
    ///
    /// **note**: does not include eyes. Use [Tee::get_all_eyes] instead
    ///
    /// # Returns
    ///
    /// A `HashMap<TeePart, &WithShadow>` containing all parts of the Tee.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::{Tee, parts::TeePart};
    ///
    /// let tee = Tee::new(/* ... */)?;
    /// let all_parts = tee.get_all_parts();
    /// let body_image = all_parts.get(&TeePart::Body).unwrap();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[instrument(level = "debug", skip(self))]
    pub fn get_all_parts(&self) -> HashMap<TeePart, &WithShadow> {
        trace!("Collecting all parts into a HashMap");
        let mut parts = HashMap::new();
        parts.insert(TeePart::Body, &self.body);
        parts.insert(TeePart::Feet, &self.feet);
        parts.insert(TeePart::Hand, &self.hand);
        debug!("Successfully collected all parts into a HashMap");
        parts
    }

    /// Returns all eye types of the Tee as a HashMap.
    ///
    /// for specific one, use [Tee::get_eye]
    ///
    /// # Returns
    ///
    /// A `HashMap<EyeType, &RgbaImage>` containing all eye types of the Tee.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use tee_morphosis::tee::{Tee, parts::EyeType};
    ///
    /// let tee = Tee::new(/* ... */)?;
    /// let all_eyes = tee.get_all_eyes();
    /// let happy_eye = all_eyes.get(&EyeType::Happy).unwrap();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[instrument(level = "debug", skip(self))]
    pub fn get_all_eyes(&self) -> HashMap<EyeType, &RgbaImage> {
        trace!("Collecting all eye types into a HashMap");
        let mut eyes = HashMap::new();
        eyes.insert(EyeType::Normal, self.get_eye(EyeType::Normal));
        eyes.insert(EyeType::Angry, self.get_eye(EyeType::Angry));
        eyes.insert(EyeType::Pain, self.get_eye(EyeType::Pain));
        eyes.insert(EyeType::Happy, self.get_eye(EyeType::Happy));
        eyes.insert(EyeType::Empty, self.get_eye(EyeType::Empty));
        eyes.insert(EyeType::Surprise, self.get_eye(EyeType::Surprise));
        debug!("Successfully collected all eye types into a HashMap");
        eyes
    }

    // Helper methods for internal use

    /// Composes all layers of the Tee onto the canvas in the correct order.
    ///
    /// # Arguments
    ///
    /// * `compose` - A closure that handles the actual composition of a layer.
    /// * `skin` - The skin layout to use for positioning.
    /// * `eye_type` - The eye type to use for the eyes.
    fn compose_layers<F>(
        &self,
        compose: &mut F,
        skin: &Skin,
        eye_type: EyeType,
    ) where
        F: FnMut(&RgbaImage, SkinPS, UVPart),
    {
        trace!("Starting to compose layers in order");

        // Layering order is important for correct appearance
        compose(&self.body.shadow, skin.body, self.used_uv.body_shadow); // body shadow
        compose(&self.feet.shadow, skin.feet_back, self.used_uv.feet_shadow); // back feet shadow
        compose(&self.feet.shadow, skin.feet, self.used_uv.feet_shadow); // front feet shadow
        compose(&self.feet.value, skin.feet_back, self.used_uv.feet); // back feet
        compose(&self.body.value, skin.body, self.used_uv.body); // body

        let eye = self.get_eye(eye_type);
        compose(eye, skin.first_eyes, self.used_uv.eyes[0]); // first eye
        compose(
            &imageops::flip_horizontal(eye),
            skin.second_eyes,
            self.used_uv.eyes[0],
        ); // second eye (flipped)

        compose(&self.feet.value, skin.feet, self.used_uv.feet); // front feet

        debug!("Successfully composed all layers");
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
///
/// # Example
///
/// ```rust,ignore
/// use tee_morphosis::tee::extract_part;
/// use image::DynamicImage;
/// use tee_morphosis::tee::uv::Part;
///
/// let img = image::open("source.png")?;
/// let part = Part { x: 10, y: 10, w: 50, h: 50 };
/// let extracted = extract_part(&img, part)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[instrument(level = "debug", skip(img), fields(part = ?part))]
fn extract_part(
    img: &DynamicImage,
    part: UVPart,
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

    trace!(
        "Extracting part at position ({}, {}) with size ({}, {})",
        part.x, part.y, part.w, part.h
    );
    let cropped_image = img.view(part.x, part.y, part.w, part.h).to_image();
    Ok(cropped_image)
}

/// Decodes image data from bytes with the specified format.
///
/// # Arguments
///
/// * `data` - The raw bytes of the image.
/// * `format` - The format of the image data.
///
/// # Returns
///
/// A `Result` which is `Ok(DynamicImage)` on successful decoding, or `Err(TeeError)` on failure.
#[instrument(level = "debug", skip(data), fields(format = ?format, data_size = data.len()))]
fn decode_image(
    data: Bytes,
    format: ImageFormat,
) -> Result<DynamicImage> {
    let mut img = ImageReader::new(Cursor::new(data));
    img.set_format(format);
    let img = img.decode()?;
    Ok(img)
}

/// Validates that the image dimensions match the expected container dimensions.
///
/// # Arguments
///
/// * `actual` - The actual dimensions of the image.
/// * `expected` - The expected dimensions of the image.
///
/// # Returns
///
/// A `Result` which is `Ok(())` if the dimensions match, or `Err(TeeError::InvalidDimensions)` if they don't.
#[instrument(level = "debug", fields(actual = ?actual, expected = ?expected))]
fn validate_image_dimensions(
    actual: (u32, u32),
    expected: (u32, u32),
) -> Result<()> {
    if actual != expected {
        error!(
            expected = ?expected,
            found = ?actual,
            "Invalid image dimensions."
        );
        return Err(TeeError::InvalidDimensions {
            expected,
            found: actual,
        });
    }
    Ok(())
}

/// Extracts a part and its shadow from the source image.
///
/// # Arguments
///
/// * `img` - The source image.
/// * `part` - The part to extract.
/// * `shadow_part` - The shadow part to extract.
///
/// # Returns
///
/// A `Result` which is `Ok(WithShadow)` containing both the part and its shadow.
#[instrument(level = "debug", skip(img), fields(part = ?part, shadow_part = ?shadow_part))]
fn extract_with_shadow(
    img: &DynamicImage,
    part: UVPart,
    shadow_part: UVPart,
) -> Result<WithShadow> {
    trace!("Extracting part and its shadow");
    let value = extract_part(img, part)?;
    let shadow = extract_part(img, shadow_part)?;
    Ok(WithShadow {
        value,
        shadow,
    })
}

/// Extracts all eye types from the source image.
///
/// # Arguments
///
/// * `img` - The source image.
/// * `eye_parts` - An array of parts for each eye type.
///
/// # Returns
///
/// A `Result` which is `Ok([EyeTypeData; 6])` containing all eye types.
#[instrument(level = "debug", skip(img, eye_parts))]
fn extract_all_eyes(
    img: &DynamicImage,
    eye_parts: &[UVPart; 6],
) -> Result<[EyeTypeData; 6]> {
    trace!("Extracting all eye types");
    let eyes = [
        EyeTypeData::Normal(extract_part(img, eye_parts[0])?),
        EyeTypeData::Angry(extract_part(img, eye_parts[1])?),
        EyeTypeData::Pain(extract_part(img, eye_parts[2])?),
        EyeTypeData::Happy(extract_part(img, eye_parts[3])?),
        EyeTypeData::Empty(extract_part(img, eye_parts[4])?),
        EyeTypeData::Surprise(extract_part(img, eye_parts[5])?),
    ];
    Ok(eyes)
}

/// Fetches an image from a URL and determines its format.
///
/// # Arguments
///
/// * `url` - The URL to fetch the image from.
///
/// # Returns
///
/// A `Result` which is `Ok((Bytes, ImageFormat))` containing the image data and its format.
#[cfg(feature = "net")]
#[instrument(level = "info", fields(url = %url))]
async fn fetch_image_from_url(url: &str) -> Result<(Bytes, ImageFormat)> {
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

    Ok((bytes, format))
}
