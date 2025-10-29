# tee_morphosis

[![Crates.io](https://img.shields.io/crates/v/tee_morphosis.svg)](https://crates.io/crates/tee_morphosis)
[![Docs.rs](https://docs.rs/tee_morphosis/badge.svg)](https://docs.rs/tee_morphosis)

A library for parsing, splitting, and building Tee skins.

This crate provides tools for parsing a source Tee skin image into its constituent parts (body, feet, hands, eyes, etc.) and then composing them into a final character image with various expressions.

## Features

- `net`: Enables network requests (loading skins from URLs) using `Tee::new_from_url`.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tee_morphosis = "0.0.1"
```

To use network capabilities (loading skins from URLs), enable the `net` feature:

```toml
[dependencies]
tee_morphosis = { version = "0.0.1", features = ["net"] }
```

## How to Use

### Example: Creating a skin from a local file

Here's a simple example of how to read a skin file, create a character with "happy" eyes, and save the result to a new file.

```rust
use std::fs;
use bytes::Bytes;
use tee_morphosis::tee::{
    Tee,
    parts::EyeType,
    uv::TEE_UV_LAYOUT,
    skin::TEE_SKIN_LAYOUT,
};
use image::ImageFormat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read your skin data from a file
    // Replace "path/to/your_skin.png" with the actual path
    let skin_data = fs::read("path/to/your_skin.png")?;

    // 2. Create a Tee instance from the image data
    let tee = Tee::new(
        Bytes::from(skin_data),
        TEE_UV_LAYOUT,
        ImageFormat::Png,
    )?;

    // 3. Compose the final image with the desired eye type
    let eye_type = EyeType::Happy;
    let image_bytes = tee.compose(
        TEE_SKIN_LAYOUT,
        eye_type,
        ImageFormat::Png,
    )?;

    // or use
    //
    // tee.compose_default(TEE_SKIN_LAYOUT)?;

    // 4. Save the result to a file
    fs::write("composed_tee.png", &image_bytes)?;

    println!("Skin successfully created and saved to composed_tee.png");

    Ok(())
}
```

### Example: Tee color rotating

Here's a simple example of how to change color with DDNet color value (or hls)

```rust
use std::fs;
use bytes::Bytes;
use tee_morphosis::tee::{
    Tee,
    hsl::ddnet_color_to_hsl,
    parts::TeePart
};
use image::ImageFormat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read your skin data from a file
    // Replace "path/to/your_skin.png" with the actual path
    let skin_data = fs::read("path/to/your_skin.png")?;

    // 2. Create a Tee instance from the image data
    let mut tee = Tee::new(
        Bytes::from(skin_data),
        TEE_UV_LAYOUT,
        ImageFormat::Png,
    )?;

    // 3. Generate hls from ddnet value
    let hsl = ddnet_color_to_hsl(1900500);
    // 4. apply to all parts
    tee.apply_hsv_to_all(hsl);
    // 4.1 or peer parts
    tee.apply_hsv_to_parts(hsl, &[TeePart::Body, TeePart::BodyShadow]);
    // 5. Now, compose it
    let composed = tee.compose_default(TEE_SKIN_LAYOUT)?;

    // 4. Save the result to a file
    fs::write("colored_composed_tee.png", &composed)?;

    println!("Skin successfully created, colored and saved to colored_composed_tee.png");

    Ok(())
}
```

### Example: Loading and creating a skin from a URL (with `net` feature)

This example demonstrates how to fetch a skin from the internet and assemble it.

```rust
use tee_morphosis::tee::{
    Tee,
    parts::EyeType,
    uv::TEE_UV_LAYOUT,
    skin::TEE_SKIN_LAYOUT,
};
use std::fs;
use image::ImageFormat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a Tee instance from a URL
    let tee = Tee::new_from_url(
        "https://teedata.net/databasev2/skins/glow_rainbow/glow_rainbow.png",
        TEE_UV_LAYOUT,
    ).await?;

    // 2. Compose the final image
    let image_bytes = tee.compose(
        TEE_SKIN_LAYOUT,
        EyeType::Surprise,
        ImageFormat::Png,
    )?;

    // 3. Save the result
    fs::write("composed_from_url.png", &image_bytes)?;

    println!("Skin successfully downloaded and saved to composed_from_url.png");

    Ok(())
}
```

## License

This project is licensed under the [MIT License](LICENSE).
