//! # Skin module

use crate::tee::uv::ContentSize;

pub type Postion = (i64, i64);
pub type Size = (u32, u32);
pub type Scale = f32;
pub type SkinPS = (Postion, Scale);

#[derive(Debug, Clone, Copy)]
/// Mappings for output
pub struct Skin {
    pub body: SkinPS,
    pub feet: SkinPS,
    pub feet_back: SkinPS,
    pub first_eyes: SkinPS,
    pub second_eyes: SkinPS,

    pub container: ContentSize,
}

// https://github.com/ddnet/ddnet-discordbot/blob/5c37e4bcc2e97347de30d48a970c75cec3ecddb3/cogs/skindb.py#L179

/// Layout for rasterized skin
pub const TEE_SKIN_LAYOUT: Skin = {
    Skin {
        body: ((16, 0), 0.66),
        feet_back: ((8, 30), 1.),
        feet: ((24, 30), 1.),
        first_eyes: ((39, 18), 0.8),
        second_eyes: ((47, 18), 0.8),
        //
        container: (96, 64),
    }
};

#[inline]
pub fn scale(
    size: Size,
    scale: f32,
) -> Size {
    (
        (size.0 as f32 * scale) as u32,
        (size.1 as f32 * scale) as u32,
    )
}
