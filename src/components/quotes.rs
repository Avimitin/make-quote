use super::{Lines, TextDrawInfo};
use image::RgbaImage;
use typed_builder::TypedBuilder;

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
        let (_, quote_height) = lines.size();
        let mut current_draw_height = (bg_height as i32 / 2) - quote_height;
        let quote_info = &quotes.quote_info;
        for line in lines {
            let x =
                centered_text_x(canvas.width(), line.width, quotes.gap) - line.first_letter_width;
            imageproc::drawing::draw_text_mut(
                &mut canvas,
                quote_info.color(),
                x,
                current_draw_height,
                quote_info.scale(),
                quote_info.font(),
                &line.text,
            );
            current_draw_height += line.height;
        }

        // Start drawing username
        let user_info = &quotes.user_info;
        let (w, _) =
            imageproc::drawing::text_size(user_info.scale(), user_info.font(), user_info.text());
        let (x, y) = (
            centered_text_x(canvas.width(), w, quotes.gap),
            (bg_height - (bg_height / 4)) as i32,
        );

        imageproc::drawing::draw_text_mut(
            &mut canvas,
            user_info.color(),
            x,
            y,
            user_info.scale(),
            user_info.font(),
            user_info.text(),
        );

        canvas
    }
}
