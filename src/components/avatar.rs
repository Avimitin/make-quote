use super::TextDrawInfo;
use image::{imageops, imageops::FilterType, Rgba, RgbaImage};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(build_method(into = RgbaImage))]
pub struct Avatar {
    img_data: RgbaImage,
    /// To produce an correct avatar image, we need the background height to resize the avatar.
    bg_height: u32,
}

impl From<Avatar> for RgbaImage {
    // Call the builder().build() method will convert Avatar into ImgBuffer
    fn from(avatar: Avatar) -> Self {
        let ratio = avatar.img_data.width() / avatar.img_data.height();
        let output_width = avatar.bg_height * ratio;

        // First let use scale the avatar to fit the background
        let mut buffer = imageops::resize(
            &avatar.img_data,
            output_width,
            avatar.bg_height,
            FilterType::CatmullRom,
        );

        // Then crop 1/4 and keep 3/4 of the avatar, this can make some dramatic view effect for the
        // final output image.
        let crop = output_width / 4;
        imageops::crop(&mut buffer, crop, 0, output_width - crop, avatar.bg_height).to_image()
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(into = RgbaImage))]
pub struct TgAvatar<'a> {
    id: u64,
    pixel: u32,
    info: TextDrawInfo<'a>,
}

const COLOR: [[u8; 4]; 7] = [
    [255, 81, 106, 255],
    [255, 168, 92, 255],
    [214, 105, 237, 255],
    [84, 203, 104, 255],
    [40, 201, 183, 255],
    [42, 158, 241, 255],
    [255, 113, 154, 255],
];

impl<'a> From<TgAvatar<'a>> for RgbaImage {
    fn from(data: TgAvatar) -> Self {
        let avatar_color = Rgba::from(COLOR[data.id as usize % 7]);

        let mut canvas = RgbaImage::from_pixel(data.pixel, data.pixel, avatar_color);
        let letter = data.info.text().to_uppercase();
        let (_, h) = imageproc::drawing::text_size(data.info.scale(), data.info.font(), &letter);
        let xy = (data.pixel as i32 / 2) - h;
        imageproc::drawing::draw_text_mut(
            &mut canvas,
            data.info.color(),
            xy,
            xy,
            data.info.scale(),
            data.info.font(),
            &letter,
        );

        canvas
    }
}
