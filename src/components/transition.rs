use image::{imageops, Rgba, RgbaImage};
use typed_builder::TypedBuilder;

/// A component for constructing a gradient overlay.
#[derive(TypedBuilder)]
#[builder(build_method(into = RgbaImage))]
pub struct Transition {
    avatar_width: u32,
    bg_height: u32,

    #[builder(default = Rgba::from([0, 0, 0, 0]))]
    starting_color: Rgba<u8>,
    #[builder(default = Rgba::from([0, 0, 0, 255]))]
    ending_color: Rgba<u8>,
}

impl From<Transition> for RgbaImage {
    // Call the Builder().build() will convert the Transition type into ImgBuffer
    fn from(trans: Transition) -> Self {
        let mut overlay = RgbaImage::new(&trans.avatar_width / 3, trans.bg_height);
        imageops::horizontal_gradient(&mut overlay, &trans.starting_color, &trans.ending_color);
        overlay
    }
}
