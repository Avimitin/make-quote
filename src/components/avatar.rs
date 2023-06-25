use super::TextDrawInfo;
use image::{imageops, imageops::FilterType, Rgba, RgbaImage};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(build_method(into = RgbaImage))]
pub struct Avatar {
    img_data: RgbaImage,
    /// To produce an correct avatar image, we need the background height to resize the avatar.
    bg_height: u32,
    #[builder(default = true)]
    enable_crop: bool,
}

impl From<Avatar> for RgbaImage {
    // Call the builder().build() method will convert Avatar into ImgBuffer
    fn from(avatar: Avatar) -> Self {
        if !avatar.enable_crop {
            return avatar.img_data;
        }

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
    bg_dim: (u32, u32),
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
        let (bg_w, bg_h) = data.bg_dim;
        let mut canvas = RgbaImage::new(bg_w / 3, bg_h);

        // First draw a circle background
        let avatar_color = Rgba::from(COLOR[data.id as usize % 7]);

        let (cv_w, cv_h) = canvas.dimensions();
        let (cv_w, cv_h) = (cv_w as i32, cv_h as i32);
        let circle_center = (cv_w / 2, cv_h / 2);
        // keep only 1/12 gaps between circle and canvas
        let radius = cv_w / 2 - cv_w / 12;
        imageproc::drawing::draw_filled_circle_mut(
            &mut canvas,
            circle_center,
            radius,
            avatar_color,
        );

        // Then draw the letter
        let info = data.info;
        let letter = info.text().to_uppercase();
        let (w, h) = imageproc::drawing::text_size(info.scale(), info.font(), &letter);
        // Adjust the font to be drawn on the center
        let (x, y) = (circle_center.0 - (w / 2), circle_center.1 - (h - h / 3));
        imageproc::drawing::draw_text_mut(
            &mut canvas,
            info.color(),
            x,
            y,
            info.scale(),
            info.font(),
            &letter,
        );

        canvas
    }
}
