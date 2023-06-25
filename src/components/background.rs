use image::{Rgba, RgbaImage};
use typed_builder::TypedBuilder;

/// This component is used for creating a background.
/// By default, it will create a background image with color RGBA(0,0,0,255),
/// width 1920 pixels and 1080 pixels.
#[derive(TypedBuilder)]
#[builder(build_method(into = RgbaImage))]
pub struct Background {
    #[builder(default=Rgba([0, 0, 0, 255]))]
    color: Rgba<u8>,
    #[builder(default=(1920, 1080))]
    output_dimension: (u32, u32),
}

impl From<Background> for RgbaImage {
    fn from(bg: Background) -> Self {
        let (width, height) = bg.output_dimension;
        RgbaImage::from_pixel(width, height, bg.color)
    }
}
