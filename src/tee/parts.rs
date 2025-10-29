//! # Module with `UV` parts

use image::RgbaImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeePart {
    Body,
    BodyShadow,
    Feet,
    FeetShadow,
    Hand,
    HandShadow,
}

/// A struct holding a part of the Tee and its corresponding shadow.
///
/// This structure allows for independent manipulation of the main part and its shadow,
/// which is useful for effects like color changes or opacity adjustments.
#[derive(Debug, Clone, PartialEq)]
pub struct WithShadow {
    /// The main part of the component
    pub value: RgbaImage,
    /// The shadow part of the component
    pub shadow: RgbaImage,
}

/// An enum representing the different states of the Tee's eyes, each holding its corresponding image.
///
/// Each variant contains the image data for that specific eye expression.
#[derive(Debug, Clone, PartialEq)]
pub enum EyeTypeData {
    /// Normal eye expression
    Normal(RgbaImage),
    /// Angry eye expression
    Angry(RgbaImage),
    /// Pain eye expression
    Pain(RgbaImage),
    /// Happy eye expression
    Happy(RgbaImage),
    /// Empty eye expression
    Empty(RgbaImage),
    /// Surprise eye expression
    Surprise(RgbaImage),
}

/// An enum to specify the desired eye state for the Tee.
///
/// This enum is used to select which eye expression to use when compositing the final image.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum EyeType {
    /// Normal eye expression
    Normal,
    /// Angry eye expression
    Angry,
    /// Pain eye expression
    Pain,
    /// Happy eye expression
    Happy,
    /// Empty eye expression
    Empty,
    /// Surprise eye expression
    Surprise,
}

impl EyeType {
    /// Returns the array index corresponding to this eye type.
    ///
    /// This is used to access the appropriate eye image in the `Tee.eye` array.
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
