//! This library provide a single function [`make_quote_image`] to turn somebody's quote into an
//! image.
//!
//! This is not an feature rich library. You may meet some draw issue. Feel free to open issue
//! at GitHub to help me improve this library. Currently the best practice is to set the output
//! size to 1920x1080.
//!
//! # Usage
//!
//! ```rust
//! use make_quote::{QuoteProducer, ImgConfig};
//!
//! // First of all, load an font into memory
//! let font = std::fs::read("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc").unwrap();
//!
//! // Create a image producer
//! let bold_font = std::fs::read("/usr/share/fonts/noto-cjk/NotoSansCJK-Bold.ttc").unwrap();
//! let light_font = include_bytes!("/usr/share/fonts/noto-cjk/NotoSansCJK-Light.ttc");
//! let producer = QuoteProducer::builder()
//!     .font(&bold_font, light_font)
//!     .output_size(1920, 1080) // optional
//!     .font_scale(120.0)       // optional
//!     .build();
//!
//! // Create image configuration
//! let config = ImgConfig::builder()
//!     .username("V5电竞俱乐部中单选手 Otto")
//!     .avatar("./assets/avatar.png")
//!     .quote("大家好，今天来点大家想看的东西。")
//!     .build();
//!
//! // Then generate the image and get the image buffer
//! let buffer = producer.make_image(&config).unwrap();
//!
//! // You can do anything you like to the buffer, save it or just send it through the net.
//! std::fs::write("./assets/test.jpg", buffer).unwrap();
//! ```
//!
//! This will generate the below output:
//!
//! <img src="https://github.com/Avimitin/make-quote/raw/master/assets/test.jpg"/>

use std::fmt::Display;
use std::io::Cursor;
use std::path::Path;

use image::imageops;
use image::{ImageError, ImageFormat};

use rusttype::Font;
use typed_builder::TypedBuilder;

mod components;

#[derive(TypedBuilder)]
pub struct QuoteProducer<'font> {
    #[builder(default = (1920, 1080), setter( transform = |width: u32, height: u32| (width, height) ))]
    output_size: (u32, u32),
    #[builder(default = 120.0)]
    font_scale: f32,
    #[builder(setter(
        transform = |bold: &'font [u8], light: &'font [u8]| {
            let bold = Font::try_from_bytes(bold).unwrap_or_else(|| panic!("invalid bold font data"));
            let light = Font::try_from_bytes(light).unwrap_or_else(|| panic!("invalid light font data"));
            FontSet {
                bold, light
            }
        }
    ))]
    font: FontSet<'font>,
}

pub struct FontSet<'font> {
    bold: Font<'font>,
    light: Font<'font>,
}

pub enum SpooledData<'data> {
    InMem(&'data [u8]),
    OnDisk(&'data Path),
    TgRandom { id: u64, name: String },
}

pub trait AsSpooledData {
    fn as_spooled_data(&self) -> SpooledData<'_>;
}

impl<P> AsSpooledData for P
where
    P: AsRef<Path>,
{
    fn as_spooled_data(&self) -> SpooledData<'_> {
        SpooledData::OnDisk(self.as_ref())
    }
}

impl AsSpooledData for str {
    fn as_spooled_data(&self) -> SpooledData<'_> {
        SpooledData::OnDisk(self.as_ref())
    }
}

impl AsSpooledData for [u8] {
    fn as_spooled_data(&self) -> SpooledData<'_> {
        SpooledData::InMem(self)
    }
}

impl<'data> AsSpooledData for SpooledData<'data> {
    fn as_spooled_data(&self) -> SpooledData<'_> {
        match self {
            SpooledData::InMem(m) => SpooledData::InMem(m),
            SpooledData::OnDisk(d) => SpooledData::OnDisk(d),
            SpooledData::TgRandom { id, name } => SpooledData::TgRandom {
                id: *id,
                name: name.to_string(),
            },
        }
    }
}

#[derive(TypedBuilder)]
pub struct ImgConfig<'a> {
    #[builder(setter( transform = |s: impl Display| s.to_string() ))]
    quote: String,
    #[builder(setter( transform = |s: impl Display| s.to_string() ))]
    username: String,
    #[builder(setter( transform = |p: &'a (impl AsSpooledData + ?Sized)| p.as_spooled_data() ))]
    avatar: SpooledData<'a>,
}

impl<'font> QuoteProducer<'font> {
    pub fn make_image(&self, config: &ImgConfig) -> Result<Vec<u8>> {
        let mut background = components::Background::builder()
            .output_dimension(self.output_size)
            .build();

        // Step 1: Overlay avatar to background
        let avatar = match &config.avatar {
            SpooledData::InMem(buffer) => {
                let img_data = image::load_from_memory(buffer)?.into_rgba8();
                components::Avatar::builder()
                    .img_data(img_data)
                    .bg_height(background.height())
                    .build()
            }
            SpooledData::OnDisk(path) => {
                let img_data = image::open(path)?.into_rgba8();
                components::Avatar::builder()
                    .img_data(img_data)
                    .bg_height(background.height())
                    .build()
            }
            SpooledData::TgRandom { id, name } => {
                let letter = name.chars().next().unwrap().to_string();
                let info = components::TextDrawInfo::builder()
                    .text(&letter)
                    .rgba([255, 255, 255, 255])
                    .scale(300.0)
                    .font(&self.font.bold)
                    .build();
                let img_data = components::TgAvatar::builder()
                    .id(*id)
                    .info(info)
                    .bg_dim(background.dimensions())
                    .build();
                components::Avatar::builder()
                    .img_data(img_data)
                    .bg_height(background.height())
                    .enable_crop(false)
                    .build()
            }
        };
        imageops::overlay(&mut background, &avatar, 0, 0);

        // Step 2: Overlay black gradient to avatar
        let gradient = components::Transition::builder()
            .avatar_width(avatar.width())
            .bg_height(background.height())
            .build();
        let offset = (avatar.width() - gradient.width()) as i64;
        imageops::overlay(&mut background, &gradient, offset, 0);

        // Step 3: Overlay quotes to background
        let quote_info = components::TextDrawInfo::builder()
            .text(&config.quote)
            .rgba([255, 255, 255, 255])
            .scale(self.font_scale)
            .font(&self.font.bold)
            .build();
        let user_info = components::TextDrawInfo::builder()
            .text(&config.username)
            .rgba([147, 147, 147, 255])
            .scale(self.font_scale / 3.0)
            .font(&self.font.light)
            .build();
        let quotes = components::Quotes::builder()
            .avatar_width(avatar.width())
            .bg_dim(background.dimensions())
            .quote_info(quote_info)
            .user_info(user_info)
            .build();
        let offset = avatar.width() as i64;
        imageops::overlay(&mut background, &quotes, offset, 0);

        let mut buffer = Cursor::new(Vec::new());
        background.write_to(&mut buffer, ImageFormat::Jpeg)?;
        Ok(buffer.into_inner())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("internal image library error: {0}")]
    ImgErr(#[from] ImageError),
    #[error("fail to read font: {0}")]
    FontErr(#[from] std::io::Error),
}

type Result<T, E = ErrorKind> = core::result::Result<T, E>;

#[test]
fn test_create_background_image() {
    use std::time::Instant;

    let bold_font = std::fs::read("/usr/share/fonts/noto-cjk/NotoSansCJK-Medium.ttc").unwrap();
    let light_font = include_bytes!("/usr/share/fonts/noto-cjk/NotoSansCJK-Light.ttc");
    let builder = QuoteProducer::builder()
        .font(&bold_font, light_font)
        .build();

    let config = ImgConfig::builder()
        .username("@V5电竞俱乐部中单选手 Otto")
        .avatar("./assets/avatar.png")
        .quote("大家好，今天来点大家想看的东西。ccccccabackajcka 阿米诺说的道理")
        .build();

    let now = Instant::now();
    let buffer = builder.make_image(&config).unwrap();
    std::fs::write("./assets/test.jpg", buffer).unwrap();
    println!("elapsed: {} ms", now.elapsed().as_millis());
    let data = SpooledData::TgRandom {
        id: 13,
        name: "ksyx".to_string(),
    };
    let config = ImgConfig::builder()
        .username("@ksyxmeow")
        .avatar(&data)
        .quote("教授可爱喵喵喵")
        .build();
    let buffer = builder.make_image(&config).unwrap();
    std::fs::write("./assets/test-tg.jpg", buffer).unwrap();
}
