pub type ContentSize = (u32, u32);

pub const BODY_SIZE: ContentSize = (96, 96);
pub const FEET_SIZE: ContentSize = (64, 32);
pub const EYE_SIZE: ContentSize = (32, 32);
pub const HAND_SIZE: ContentSize = (32, 32);

#[derive(Debug, Clone, Copy)]
pub struct Part {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Copy)]
/// Mappings for parsing
pub struct UV {
    pub body: Part,
    pub body_shadow: Part,
    pub feet: Part,
    pub feet_shadow: Part,
    pub hand: Part,
    pub hand_shadow: Part,
    /// [Normal, Angry, Pain, Happy, Empty, Surprise]
    pub eyes: [Part; 6],
    pub container: ContentSize,
}

/// Describe position and size of each part of Tee on the image (256x128).
pub const TEE_UV_LAYOUT: UV = {
    const BODY_END_X: u32 = BODY_SIZE.0;
    const HANDS_AND_FEET_START_X: u32 = BODY_SIZE.0 * 2;
    const HAND_SHADOW_START_X: u32 = HANDS_AND_FEET_START_X + HAND_SIZE.0;
    const FEET_START_Y: u32 = HAND_SIZE.1;
    const FEET_SHADOW_START_Y: u32 = FEET_START_Y + FEET_SIZE.1;
    const EYES_START_X: u32 = 64;
    const EYES_START_Y: u32 = BODY_SIZE.1;

    UV {
        container: (256, 128),
        body: Part {
            x: 0,
            y: 0,
            w: BODY_SIZE.0,
            h: BODY_SIZE.1,
        },
        body_shadow: Part {
            x: BODY_END_X,
            y: 0,
            w: BODY_SIZE.0,
            h: BODY_SIZE.1,
        },
        feet: Part {
            x: HANDS_AND_FEET_START_X,
            y: FEET_START_Y,
            w: FEET_SIZE.0,
            h: FEET_SIZE.1,
        },
        feet_shadow: Part {
            x: HANDS_AND_FEET_START_X,
            y: FEET_SHADOW_START_Y,
            w: FEET_SIZE.0,
            h: FEET_SIZE.1,
        },
        hand: Part {
            x: HANDS_AND_FEET_START_X,
            y: 0,
            w: HAND_SIZE.0,
            h: HAND_SIZE.1,
        },
        hand_shadow: Part {
            x: HAND_SHADOW_START_X,
            y: 0,
            w: HAND_SIZE.0,
            h: HAND_SIZE.1,
        },
        eyes: [
            Part {
                x: EYES_START_X,
                y: EYES_START_Y,
                w: EYE_SIZE.0,
                h: EYE_SIZE.1,
            }, // Normal
            Part {
                x: EYES_START_X + EYE_SIZE.0,
                y: EYES_START_Y,
                w: EYE_SIZE.0,
                h: EYE_SIZE.1,
            }, // Angry
            Part {
                x: EYES_START_X + EYE_SIZE.0 * 2,
                y: EYES_START_Y,
                w: EYE_SIZE.0,
                h: EYE_SIZE.1,
            }, // Pain
            Part {
                x: EYES_START_X + EYE_SIZE.0 * 3,
                y: EYES_START_Y,
                w: EYE_SIZE.0,
                h: EYE_SIZE.1,
            }, // Happy
            Part {
                x: EYES_START_X + EYE_SIZE.0 * 4,
                y: EYES_START_Y,
                w: EYE_SIZE.0,
                h: EYE_SIZE.1,
            }, // Empty
            Part {
                x: EYES_START_X + EYE_SIZE.0 * 5,
                y: EYES_START_Y,
                w: EYE_SIZE.0,
                h: EYE_SIZE.1,
            }, // Surprise
        ],
    }
};
