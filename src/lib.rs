//! This library provide a single function [`make_quote_image`] to turn somebody's quote into an
//! image. Example image can be viewed at GitHub repository.
//!
//! This is not an feature rich library. You may meet some draw issue. Feel free to open issue
//! at GitHub to help me improve this library. Currently the best practice is to set the output
//! size to 1920x1080.

use std::fmt::Display;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::{
    imageops::{self, FilterType},
    ImageBuffer, ImageError, ImageFormat, Pixel, Rgba, RgbaImage,
};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use typed_builder::TypedBuilder;

/// Configuration describe how to genrate the image.
#[derive(TypedBuilder)]
pub struct Configuration<'font> {
    #[builder(setter( transform = |width: u32, height: u32| (width, height) ))]
    output_size: (u32, u32),
    #[builder(setter( transform = |s: impl Display| s.to_string() ))]
    quote: String,
    #[builder(setter( transform = |s: impl Display| s.to_string() ))]
    username: String,

    #[builder(setter( transform = |p: impl AsRef<Path>| p.as_ref().to_path_buf() ))]
    avatar_path: PathBuf,

    font: &'font Font<'static>,

    #[builder(default, setter(strip_option))]
    quote_font_scale: Option<f32>,
    #[builder(default, setter(strip_option))]
    username_font_scale: Option<f32>,
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("internal image library error: {0}")]
    ImgErr(#[from] ImageError),
    #[error("fail to read font: {0}")]
    FontErr(#[from] std::io::Error),
}

type Result<T, E = ErrorKind> = core::result::Result<T, E>;

/// Alias to the RGBA image buffer type
type RgbaImgBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;

/// Make an image base on somebody's quote. Return bytes encoded in JPEG format.
pub fn make_quote_image(config: &Configuration) -> Result<Vec<u8>> {
    let black = Rgba([0, 0, 0, 255]);
    let (bg_width, bg_height) = config.output_size;

    let mut background = RgbaImage::from_pixel(bg_width, bg_height, black);
    let avatar = make_avatar(config)?;
    let gradient = make_gradient_overlay(config, avatar.width());

    // Step 1: Overlay avatar to background
    imageops::overlay(&mut background, &avatar, 0, 0);

    // Step 2: Overlay black gradient to avatar
    let offset = (avatar.width() - gradient.width()) as i64;
    imageops::overlay(&mut background, &gradient, offset, 0);

    // Step 3: Draw font on background
    draw_quote(&mut background, config, avatar.width())?;

    let mut buffer = Cursor::new(Vec::new());
    background.write_to(&mut buffer, ImageFormat::Jpeg)?;
    Ok(buffer.into_inner())
}

/// Split long string to multiline
fn split_quotes(quote: &str) -> Vec<String> {
    let max_length = 12;
    quote
        .lines()
        .flat_map(|line| {
            let chars = line.chars().collect::<Vec<_>>();
            chars
                .chunks(max_length)
                .map(|chk| chk.iter().collect::<String>())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<String>>()
}

/// Draw quote on background
fn draw_quote(bg: &mut RgbaImgBuf, config: &Configuration, avatar_width: u32) -> Result<()> {
    let font = &config.font;
    let white = Rgba([255, 255, 255, 255]);
    let gray = Rgba([147, 147, 147, 255]);
    let (bg_width, bg_height) = config.output_size;
    let quote_font_scale = config.quote_font_scale.unwrap_or(120.0);
    let username_font_scale = config.username_font_scale.unwrap_or(80.0);
    let quote_text_scale = Scale::uniform(quote_font_scale);
    let username_text_scale = Scale::uniform(username_font_scale);

    let quote_lines = split_quotes(&config.quote);
    let (quote_text_width, quote_text_height) =
        imageproc::drawing::text_size(quote_text_scale, font, &quote_lines[0]);

    let blank_width = bg_width - avatar_width;
    let text_gap = blank_width as i32 - quote_text_width;
    let text_draw_x_offset: i32 = avatar_width as i32 + (text_gap / 2);
    let mut text_draw_y_offset: i32 = (bg_height as i32 / 3) - quote_text_height;

    for quote in split_quotes(&config.quote) {
        draw_text_mut(
            bg,
            white,
            text_draw_x_offset,
            text_draw_y_offset,
            quote_text_scale,
            font,
            &quote,
        );

        text_draw_y_offset += quote_font_scale as i32;
    }

    let (usr_text_width, _) =
        imageproc::drawing::text_size(username_text_scale, font, &config.username);
    let text_draw_x_offset = (text_draw_x_offset + quote_text_width / 2) - usr_text_width / 2;
    draw_text_mut(
        bg,
        gray,
        text_draw_x_offset,
        text_draw_y_offset + (quote_font_scale as i32),
        username_text_scale,
        font,
        &format!("– {}", config.username),
    );

    Ok(())
}

// TODO: return Result
/// Scale and crop the avatar to fit the background.
fn make_avatar(cfg: &Configuration) -> Result<RgbaImgBuf> {
    let buffer = image::open(&cfg.avatar_path)?.into_rgba8();

    let ratio = buffer.width() as f32 / buffer.height() as f32;
    let bg_height = cfg.output_size.1;
    let new_width = (bg_height as f32 * ratio) as u32;

    // scale avatar size to background height
    let mut buffer = imageops::resize(&buffer, new_width, bg_height, FilterType::Nearest);

    // crop 1/4 from left
    let keep_width = buffer.width() - (buffer.width() / 4);
    Ok(imageops::crop(&mut buffer, new_width / 4, 0, keep_width, bg_height).to_image())
}

/// Create a transparent to black gradient overlay
fn make_gradient_overlay(cfg: &Configuration, avatar_width: u32) -> RgbaImgBuf {
    let mut gradient_overlay = RgbaImage::new(avatar_width / 3, cfg.output_size.1);
    let start = Rgba::from_slice(&[0, 0, 0, 0]);
    let end = Rgba::from_slice(&[0, 0, 0, 255]);
    imageops::horizontal_gradient(&mut gradient_overlay, start, end);
    gradient_overlay
}

/// Read a font file into memory
pub fn load_font<P: AsRef<Path>>(p: P) -> Result<Font<'static>> {
    let file = std::fs::read(p)?;
    let font = Font::try_from_vec(file);
    if font.is_none() {
        return Err(ErrorKind::FontErr(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid font",
        )));
    }

    Ok(font.unwrap())
}

#[test]
fn test_create_background_image() {
    use std::time::Instant;

    let font = load_font("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc").unwrap();

    let config = Configuration::builder()
        .output_size(1920, 1080)
        .font(&font)
        .avatar_path("./assets/avatar.png")
        .quote("大家好，今天来点大家想看的东西。")
        .username("V5电竞俱乐部中单选手 Otto")
        .build();

    let now = Instant::now();
    let buffer = make_quote_image(&config).unwrap();
    std::fs::write("./assets/test.jpg", buffer).unwrap();
    println!("elapsed: {} ms", now.elapsed().as_millis());
}
