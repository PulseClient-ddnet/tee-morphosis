pub type Postion = (i64, i64);
pub type Size = (u32, u32);
pub type SkinPS = (Postion, Size);

#[derive(Debug, Clone, Copy)]
/// Mappings for output
pub struct Skin {
    pub body: SkinPS,
    pub front_feet: SkinPS,
    pub back_feet: SkinPS,
    // pub hand: Part,
    // pub hand_shadow: Part,
    pub first_eyes: SkinPS,
    pub second_eyes: SkinPS,

    pub container: (u32, u32),
}

/// Layout for rasterized skin
pub const TEE_SKIN_LAYOUT: Skin = {
    Skin {
        back_feet: ((8, 32), (64, 30)),
        body: ((14, 0), (64, 64)),
        front_feet: ((24, 32), (64, 30)),
        first_eyes: ((36, 17), (26, 26)),
        second_eyes: ((45, 17), (26, 26)),
        //
        container: (96, 64),
    }
};
