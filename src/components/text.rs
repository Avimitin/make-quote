use image::Rgba;
use rusttype::Font;
use typed_builder::TypedBuilder;

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

    pub fn text(&self) -> &str {
        self.text
    }

    pub fn color(&self) -> Rgba<u8> {
        self.rgba
    }

    pub fn scale(&self) -> rusttype::Scale {
        self.scale
    }

    pub fn font(&self) -> &Font<'_> {
        self.font
    }
}

pub struct Lines {
    // line text, line width, line height
    data: Vec<(String, i32, i32)>,
    // Total required space width and height
    size: (i32, i32),
}

impl std::iter::IntoIterator for Lines {
    type Item = (String, i32, i32);

    type IntoIter = std::vec::IntoIter<(String, i32, i32)>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a Lines {
    type Item = &'a (String, i32, i32);

    type IntoIter = std::slice::Iter<'a, (String, i32, i32)>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl Lines {
    pub fn new(info: &TextDrawInfo<'_>, limit: i32) -> Self {
        let mut lines = Vec::new();
        let mut buffer = String::new();
        let (mut max_w, mut max_h) = (0, 0);
        // TODO: This is inefficient, guess and step with multiple characters
        let (mut last_w, mut last_h) = (0, 0);
        for char in info.text.chars() {
            buffer.push(char);

            let (w, h) = imageproc::drawing::text_size(info.scale, info.font, &buffer);
            last_w = w;
            last_h = h;
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
        lines.push((buffer, last_w, last_h));
        max_w = std::cmp::max(max_w, last_w);
        max_h += last_h;

        Self {
            data: lines,
            size: (max_w, max_h),
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }
}
