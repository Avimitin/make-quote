use image::{
    imageops::{self, FilterType},
    ImageBuffer, Pixel, Rgba, RgbaImage,
};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};

pub struct Configuration {
    output_size: (u32, u32),
    quote: String,
    username: String,
    avatar_path: String,
    font_path: String,
    output_path: String,
}

pub type RgbaImgBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub fn make_quote_image(config: &Configuration) {
    let black = Rgba([0, 0, 0, 255]);
    let (bg_width, bg_height) = config.output_size;

    let mut background = RgbaImage::from_pixel(bg_width, bg_height, black);
    let avatar = make_avatar(config);
    let gradient = make_gradient_overlay(config, avatar.width());

    // Step 1: Overlay avatar to background
    imageops::overlay(&mut background, &avatar, 0, 0);

    // Step 2: Overlay black gradient to avatar
    let offset = (avatar.width() - gradient.width()) as i64;
    imageops::overlay(&mut background, &gradient, offset, 0);

    // Step 3: Draw font on background
    draw_quote(&mut background, config, avatar.width());

    // Step Final: Save
    background.save(&config.output_path).unwrap();
}

fn draw_quote(bg: &mut RgbaImgBuf, config: &Configuration, avatar_width: u32) {
    let font = read_font(config);
    let white = Rgba([255, 255, 255, 255]);
    let gray = Rgba([147, 147, 147, 255]);
    let (bg_width, bg_height) = config.output_size;
    let quote_text_scale = Scale::uniform(120.0);
    let username_text_scale = Scale::uniform(80.0);
    let (quote_text_width, quote_text_height) =
        imageproc::drawing::text_size(quote_text_scale, &font, &config.quote);
    let blank_width = bg_width - avatar_width;
    let text_gap = blank_width as i32 - quote_text_width;
    let text_draw_x_offset: i32 = avatar_width as i32 + (text_gap / 2);
    let text_draw_y_offset: i32 = (bg_height as i32 / 2) - quote_text_height;
    draw_text_mut(
        bg,
        white,
        text_draw_x_offset,
        text_draw_y_offset,
        quote_text_scale,
        &font,
        &config.quote,
    );

    let (usr_text_width, _) =
        imageproc::drawing::text_size(username_text_scale, &font, &config.username);
    let text_draw_x_offset = (text_draw_x_offset + quote_text_width / 2) - usr_text_width / 2;
    draw_text_mut(
        bg,
        gray,
        text_draw_x_offset,
        text_draw_y_offset + (quote_text_scale.x as i32),
        username_text_scale,
        &font,
        &format!("– {}", config.username),
    );
}

// TODO: return Result
fn make_avatar(cfg: &Configuration) -> RgbaImgBuf {
    let buffer = image::open(&cfg.avatar_path).unwrap().into_rgba8();

    let ratio = buffer.width() as f32 / buffer.height() as f32;
    let bg_height = cfg.output_size.1;
    let new_width = (bg_height as f32 * ratio) as u32;

    // scale avatar size to background height
    let mut buffer = imageops::resize(&buffer, new_width, bg_height, FilterType::Nearest);

    // crop 1/4 from left
    let keep_width = buffer.width() - (buffer.width() / 4);
    imageops::crop(&mut buffer, new_width / 4, 0, keep_width, bg_height).to_image()
}

fn make_gradient_overlay(cfg: &Configuration, avatar_width: u32) -> RgbaImgBuf {
    let mut gradient_overlay = RgbaImage::new(avatar_width / 3, cfg.output_size.1);
    let start = Rgba::from_slice(&[0, 0, 0, 0]);
    let end = Rgba::from_slice(&[0, 0, 0, 255]);
    imageops::horizontal_gradient(&mut gradient_overlay, start, end);
    gradient_overlay
}

// TODO: return Result
fn read_font(cfg: &Configuration) -> Font {
    let file = std::fs::read(&cfg.font_path).unwrap();
    Font::try_from_vec(file).unwrap()
}

#[test]
fn test_create_background_image() {
    use std::time::Instant;

    let config = Configuration {
        output_size: (1920, 1080),
        output_path: "./assets/test.jpg".to_string(),
        font_path: "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc".to_string(),
        avatar_path: "./assets/avatar.jpg".to_string(),
        quote: "我超，OP！".to_string(),
        username: "嘉然今天吃什么".to_string(),
    };

    let now = Instant::now();
    make_quote_image(&config);
    println!("elapsed: {} ms", now.elapsed().as_millis());
}