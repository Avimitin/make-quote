use image::{
    imageops::{self, FilterType},
    Rgba, RgbaImage,
};
use rusttype::Font;
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

#[derive(TypedBuilder)]
#[builder(build_method(into = RgbaImage))]
pub struct Quotes<'a> {
    #[builder(default = 30)]
    gap: u32,
    avatar_width: u32,
    bg_dim: (u32, u32),

    quote_info: TextDrawInfo<'a>,
    user_info: TextDrawInfo<'a>,
}

#[derive(TypedBuilder)]
pub struct TextDrawInfo<'a> {
    text: &'a str,
    #[builder(setter(transform = |s: impl Into<Rgba<u8>>| s.into()))]
    rgba: Rgba<u8>,
    #[builder(setter(transform = |s: f32| rusttype::Scale::uniform(s)))]
    scale: rusttype::Scale,
    font: &'a Font<'a>,
}

impl<'a> TextDrawInfo<'a> {
    pub fn raw_scale_factor(&self) -> f32 {
        self.scale.x
    }
}

struct Lines {
    // line text, line width, line height
    data: Vec<(String, i32, i32)>,
    // Total required space width and height
    size: (i32, i32),
}

impl Lines {
    fn new(info: &TextDrawInfo<'_>, limit: i32) -> Self {
        let mut lines = Vec::new();
        let mut buffer = String::new();
        let (mut max_w, mut max_h) = (0, 0);
        // TODO: This is inefficient, guess and step with multiple characters
        let mut last = (0, 0);
        for char in info.text.chars() {
            buffer.push(char);

            let (w, h) = imageproc::drawing::text_size(info.scale, info.font, &buffer);
            last = (w, h);
            if w >= limit {
                // if adding a new character will exceed the limit, used the character before
                let c = buffer.chars().count();
                let s = buffer.chars().take(c - 1).collect::<String>();
                let data = (s, w, h);
                lines.push(data);
                buffer.clear();
                buffer.push(char);

                max_w = std::cmp::max(max_w, w);
                max_h += h;
            }
        }
        lines.push((buffer, last.0, last.1));

        Self {
            data: lines,
            size: (max_w, max_h),
        }
    }
}

//                                                          The X
// <--                 background width                    -->|
// <- - half width          ->|
// <-        + other factor    ->|
// <-    - w / 2  ->|
fn centered_text_x(bg_w: u32, text_w: i32, other_factor: u32) -> i32 {
    (bg_w as i32 / 2) + (other_factor as i32) - (text_w / 2)
}

impl<'a> From<Quotes<'a>> for RgbaImage {
    fn from(quotes: Quotes<'a>) -> Self {
        // First let use calculate the quote text size
        let (bg_width, bg_height) = quotes.bg_dim;
        let quote_area_width = bg_width - quotes.avatar_width;
        let mut canvas = RgbaImage::new(quote_area_width, bg_height);
        let max_text_draw_width = canvas.width() - (quotes.gap * 2);

        // Then start drawing quotes
        let lines = Lines::new(&quotes.quote_info, max_text_draw_width as i32);
        let mut current_draw_height = (bg_height as i32 / 2) - lines.size.1;
        let quote_info = &quotes.quote_info;
        for (text, width, height) in lines.data {
            let x = centered_text_x(canvas.width(), width, quotes.gap);
            imageproc::drawing::draw_text_mut(
                &mut canvas,
                quote_info.rgba,
                x,
                current_draw_height,
                quote_info.scale,
                quote_info.font,
                &text,
            );
            current_draw_height += height;
        }

        // Start drawing username
        let user_info = &quotes.user_info;
        let (w, _) = imageproc::drawing::text_size(user_info.scale, user_info.font, user_info.text);
        let (x, y) = (
            (canvas.width() / 2 - quotes.gap - (w as u32 / 2)) as i32,
            (bg_height - (bg_height / 4)) as i32,
        );

        imageproc::drawing::draw_text_mut(
            &mut canvas,
            user_info.rgba,
            x,
            y,
            user_info.scale,
            user_info.font,
            user_info.text,
        );

        canvas
    }
}
