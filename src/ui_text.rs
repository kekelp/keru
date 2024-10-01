use glyphon::{Buffer as GlyphonBuffer, Color, TextArea, TextBounds};


#[derive(Clone, Debug)]
pub struct TextAreaParams {
    pub left: f32,
    pub top: f32,
    pub scale: f32,
    pub bounds: TextBounds,
    pub default_color: Color,
    pub last_frame_touched: u64,
    pub last_hash: u64,
}

pub struct FullText {
    pub buffer: GlyphonBuffer,
    pub params: TextAreaParams,
}

// Lots of terrible code here, but I blame Glyphon.

pub struct TextAreaIter<'a> {
    data: &'a [FullText],
    current_index: usize,
}

impl<'a> TextAreaIter<'a> {
    fn new(data: &'a [FullText]) -> Self {
        Self {
            data,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for TextAreaIter<'a> {
    type Item = TextArea<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.data.len() {
            return None;
        }

        let item = &self.data[self.current_index];

        let text_area = TextArea {
            buffer: &item.buffer,
            left: item.params.left,
            top: item.params.top,
            scale: item.params.scale,
            bounds: item.params.bounds,
            default_color: item.params.default_color,
            custom_glyphs: &[],
        };

        self.current_index += 1;
        return Some(text_area);
    }
}

pub fn render_iter<'a>(data: &'a Vec<FullText>) -> TextAreaIter<'a> {
    return TextAreaIter::new(data);
}
