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
    data: Vec<Line>,
    // Total required space width and height
    size: (i32, i32),
}

pub struct Line {
    pub text: String,
    pub width: i32,
    pub height: i32,
    pub first_char_width: i32,
}

impl std::iter::IntoIterator for Lines {
    type Item = Line;

    type IntoIter = std::vec::IntoIter<Line>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a Lines {
    type Item = &'a Line;

    type IntoIter = std::slice::Iter<'a, Line>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl Lines {
    pub fn new(info: &TextDrawInfo<'_>, limit: i32) -> Self {
        let mut lines = Vec::new();
        let mut buffer = String::new();
        let (mut text_area_w, mut text_area_h) = (0, 0);
        let total = info.text.chars().count();

        // TODO: This is inefficient, guess and step with multiple characters
        for (idx, char) in info.text.chars().enumerate() {
            buffer.push(char);

            let (line_w, line_h) = imageproc::drawing::text_size(info.scale, info.font, &buffer);

            let drop_needed = line_w >= limit || char == '\n';
            let match_newline = drop_needed || idx == total - 1;
            if match_newline {
                let new_line = if drop_needed {
                    let n = buffer.chars().count();
                    let s = buffer.chars().take(n - 1).collect::<String>();
                    buffer.clear();
                    // we need to put the char back to next line, except the '\n' character.
                    if line_w >= limit {
                        buffer.push(char);
                    }

                    s
                } else {
                    buffer.to_string()
                };

                let (fcw, _) = imageproc::drawing::text_size(
                    info.scale,
                    info.font,
                    &new_line.chars().next().unwrap().to_string(),
                );
                lines.push(Line {
                    text: new_line,
                    width: line_w,
                    height: line_h,
                    first_char_width: fcw,
                });

                text_area_w = std::cmp::max(text_area_w, line_w);
                text_area_h += line_h;
            }
        }

        Self {
            data: lines,
            size: (text_area_w, text_area_h),
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }
}
