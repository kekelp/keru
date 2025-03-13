use glyphon::{Color as GlyphonColor, TextBounds, Viewport, TextArea};
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Family, FontSystem, Metrics, Shaping, SwashCache,
    TextAtlas, TextRenderer,
};

// another stupid sub struct for dodging partial borrows
pub(crate) struct TextSystem {
    pub font_system: FontSystem,
    pub cache: SwashCache,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_areas: Vec<FullText>,
    pub glyphon_viewport: Viewport,
}
const GLOBAL_TEXT_METRICS: Metrics = Metrics::new(24.0, 24.0);


impl TextSystem {
    pub(crate) fn maybe_new_text_area(
        &mut self,
        text: Option<&str>,
        current_frame: u64,
    ) -> Option<usize> {
        let text = match text {
            Some(text) => text,
            None => return None,
        };

        let mut buffer = GlyphonBuffer::new(&mut self.font_system, GLOBAL_TEXT_METRICS);
        buffer.set_size(&mut self.font_system, Some(500.), Some(500.));

        for line in &mut buffer.lines {
            line.set_align(Some(glyphon::cosmic_text::Align::Center));
        }

        // todo: maybe remove duplication with set_text_hashed (the branch in refresh_node that updates the text without creating a new entry here)
        // buffer.set_wrap(&mut self.font_system, glyphon::Wrap::Word);
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        let params = TextAreaParams {
            left: 10.0,
            top: 10.0,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: 10000,
                bottom: 10000,
            },
            default_color: GlyphonColor::rgb(255, 255, 255),
            last_frame_touched: current_frame,
        };
        self.text_areas.push(FullText { buffer, params });
        let text_id = self.text_areas.len() - 1;

        return Some(text_id);
    }

    pub(crate) fn refresh_last_frame(&mut self, text_id: Option<usize>, current_frame: u64) {
        if let Some(text_id) = text_id {
            self.text_areas[text_id].params.last_frame_touched = current_frame;
        }
    }
}


#[derive(Clone, Debug)]
pub struct TextAreaParams {
    pub left: f32,
    pub top: f32,
    pub scale: f32,
    pub bounds: TextBounds,
    pub default_color: GlyphonColor,
    pub last_frame_touched: u64,
}

pub struct FullText {
    pub buffer: GlyphonBuffer,
    pub params: TextAreaParams,
}

// Lots of terrible code here, but I blame Glyphon.

pub struct TextAreaIter<'a> {
    data: &'a [FullText],
    frame: u64,
    current_index: usize,
}

impl<'a> TextAreaIter<'a> {
    fn new(data: &'a [FullText], frame: u64) -> Self {
        Self {
            data,
            frame,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for TextAreaIter<'a> {
    type Item = TextArea<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {           
            if self.current_index >= self.data.len() {
                return None;
            }
                
            let item = &self.data[self.current_index];
            self.current_index += 1;
            
            if item.params.last_frame_touched == self.frame {

                let text_area = TextArea {
                    buffer: &item.buffer,
                    left: item.params.left,
                    top: item.params.top,
                    scale: item.params.scale,
                    bounds: item.params.bounds,
                    default_color: item.params.default_color,
                    custom_glyphs: &[],
                };
                
                return Some(text_area);
            }
        }
    }
}

pub fn render_iter<'a>(data: &'a Vec<FullText>, frame: u64) -> TextAreaIter<'a> {
    return TextAreaIter::new(data, frame);
}
