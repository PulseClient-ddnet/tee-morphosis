#![cfg_attr(docsrs, feature(doc_cfg))]

//! # tee_morphosis
//!
//! This crate for parsing, splitting and building Tees
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
//! ## available features:
//! - `net`: include tokio for [Tee::new_from_url]

pub mod error;
pub mod tee;

#[cfg(doc)]
use tee::Tee;
#[cfg(doc)]
use tee::skin::Skin;
#[cfg(doc)]
use tee::uv::UV;
