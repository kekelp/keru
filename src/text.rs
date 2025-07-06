use parley2::StyleHandle;

use crate::*;

#[derive(Debug)]
pub enum TextI {
    TextBox(parley2::TextBoxHandle),
    _StaticTextBox(parley2::StaticTextBoxHandle),
    TextEdit(parley2::TextEditHandle),
}

impl Ui {
    pub(crate) fn set_text(&mut self, i: NodeI, edit: bool, text: &str, style: Option<&StyleHandle>) -> &mut Self {
        match &self.nodes[i].text_i {
            Some(TextI::TextBox(handle)) => {
                if edit {
                    // Switch from TextBox to TextEdit
                    let new_handle = self.sys.text.add_text_edit(text.to_string(), (0.0, 0.0), (500.0, 500.0), 0.5);
                    
                    if let Some(style) = style {
                        self.sys.text.get_text_edit_mut(&new_handle).set_style(style);
                    }

                    self.nodes[i].text_i = Some(TextI::TextEdit(new_handle));
                } else {

                    if let Some(style) = style {
                        self.sys.text.get_text_box_mut(&handle).set_style(style);
                    }

                    *self.sys.text.get_text_box_mut(&handle).raw_text_mut() = text.to_string();
                }
            },
            Some(TextI::TextEdit(_handle)) => {
                if edit {
                    // do nothing
                } else {
                    // Switch from TextEdit to TextBox
                    let new_handle = self.sys.text.add_text_box(text.to_string(), (0.0, 0.0), (500.0, 500.0), 0.5);
                    self.nodes[i].text_i = Some(TextI::TextBox(new_handle));
                }
            },
            Some(_) => {}
            None => {
                if edit {
                    let new_handle = self.sys.text.add_text_edit(text.to_string(), (0.0, 0.0), (500.0, 500.0), 0.5);
                    self.nodes[i].text_i = Some(TextI::TextEdit(new_handle));
                } else {
                    let new_handle = self.sys.text.add_text_box(text.to_string(), (0.0, 0.0), (500.0, 500.0), 0.5);

                    if let Some(style) = style {
                        self.sys.text.get_text_box_mut(&new_handle).set_style(style);
                    }

                    self.nodes[i].text_i = Some(TextI::TextBox(new_handle));
                }
            },
        }

        self.push_text_change(i);
        
        return self;
    }

    /// Insert a style, and get a [`StyleHandle`] that can be used to access and mutate it with the [`Self::get_style_mut`] functions.
    /// 
    /// This function **should not be called on every frame**, as that would insert a new copy of the style every time.
    /// 
    // todo: figure out a better way to do this.  
    pub fn insert_style(&mut self, style: TextStyle) -> StyleHandle {
        self.sys.text.add_style(style)
    }

    pub fn get_style(&self, style: &StyleHandle) -> &TextStyle {
        self.sys.text.get_style(style)
    }

    pub fn get_style_mut(&mut self, style: &StyleHandle) -> &mut TextStyle {
        self.sys.text.get_style_mut(style)
    }
}

// impl TextSystem {
//     pub(crate) fn new_text_area(
//         &mut self,
//         text: &str,
//         edit: bool,
//         current_frame: u64,
//     ) -> TextI {

//         let mut buffer = GlyphonBuffer::new(&mut self.font_system, GLOBAL_TEXT_METRICS);
//         buffer.set_size(&mut self.font_system, Some(500.), Some(500.));

//         for line in &mut buffer.lines {
//             line.set_align(Some(glyphon::cosmic_text::Align::Center));
//         }

//         // todo: maybe remove duplication with set_text_hashed (the branch in refresh_node that updates the text without creating a new entry here)
//         // buffer.set_wrap(&mut self.font_system, glyphon::Wrap::Word);
//         buffer.set_text(
//             &mut self.font_system,
//             text,
//             Attrs::new().family(Family::SansSerif),
//             Shaping::Advanced,
//         );

//         let params = TextAreaParams {
//             left: 10.0,
//             top: 10.0,
//             scale: 1.0,
//             bounds: TextBounds {
//                 left: 0,
//                 top: 0,
//                 right: 10000,
//                 bottom: 10000,
//             },
//             default_color: GlyphonColor::rgb(255, 255, 255),
//             last_frame_touched: current_frame,
//         };

//         let text_i;
//         if edit {
//             buffer.set_text(
//                 &mut self.font_system,
//                 "Default text or something",
//                 Attrs::new().family(Family::SansSerif),
//                 Shaping::Advanced,
//             );
//             let editor = Editor::new(buffer);
//             let history = TextEditHistory::new();
//             let i = self.slabs.editors.insert(FullTextEdit { editor, params, history });
//             text_i = TextI::TextEditI(i);
//         } else {
//             self.slabs.boxes.push(FullText { buffer, params });
//             let i = self.slabs.boxes.len() - 1;
//             text_i = TextI::TextI(i);

//         }

//         return text_i;
//     }


//     pub(crate) fn refresh_last_frame(&mut self, text_i: Option<TextI>, current_frame: u64) {
//         if let Some(text_i) = text_i {
//             self.slabs.text_or_textedit_params(text_i).last_frame_touched = current_frame;
//         }
//     }

// }

// // Lots of terrible code here, but I blame Glyphon.

// pub(crate) trait RipOutTheBuffer {
//     fn buffer_mut(&mut self) -> &mut GlyphonBuffer;
//     fn buffer(&self) -> &GlyphonBuffer;
// }
// impl RipOutTheBuffer for Editor<'static> {
//     fn buffer_mut(&mut self) -> &mut GlyphonBuffer {
//         let buffer_ref = self.buffer_ref_mut();
//         match buffer_ref {
//             glyphon::cosmic_text::BufferRef::Owned(buffer) => {
//                 return buffer;
//             },
//             _ => panic!("We don't do that")
//         }
//     }
//     fn buffer(&self) -> &GlyphonBuffer {
//         let buffer_ref = self.buffer_ref();
//         match buffer_ref {
//             glyphon::cosmic_text::BufferRef::Owned(buffer) => {
//                 return buffer;
//             },
//             _ => panic!("We don't do that")
//         }
//     }

// }
// trait PutItBackTogether {
//     fn glyphon_text_area(&mut self) -> TextArea<'_>;
// }
// impl PutItBackTogether for FullText {
//     fn glyphon_text_area(&mut self) -> TextArea<'_> {
//         return TextArea {
//             buffer: &self.buffer,
//             left: self.params.left,
//             top: self.params.top,
//             scale: self.params.scale,
//             bounds: self.params.bounds,
//             default_color: self.params.default_color,
//             custom_glyphs: &[],
//         };
//     }
// }
// impl PutItBackTogether for FullTextEdit {
//     fn glyphon_text_area(&mut self) -> TextArea<'_> {
//         return TextArea {
//             buffer: self.editor.buffer_mut(),
//             left: self.params.left,
//             top: self.params.top,
//             scale: self.params.scale,
//             bounds: self.params.bounds,
//             default_color: self.params.default_color,
//             custom_glyphs: &[],
//         };
//     }
// }

// impl TextSlabs {
//     pub(crate) fn text_or_textedit_buffer(&mut self, text_i: TextI) -> &mut glyphon::Buffer {
//         match text_i {
//             TextI::TextI(text_i) => {
//                 return &mut self.boxes[text_i].buffer;
//             }
//             TextI::TextEditI(text_i) => {
//                 return self.editors[text_i].editor.buffer_mut(); 
//             },
//         }
//     }

//     pub(crate) fn text_or_textedit_params(&mut self, text_i: TextI) -> &mut TextAreaParams {
//         match text_i {
//             TextI::TextI(text_i) => {
//                 return &mut self.boxes[text_i].params;
//             }
//             TextI::TextEditI(text_i) => {
//                 return &mut self.editors[text_i].params;
//             },
//         }
//     } 
// }

// #[derive(Clone, Debug)]
// pub struct TextAreaParams {
//     pub left: f32,
//     pub top: f32,
//     pub scale: f32,
//     pub bounds: TextBounds,
//     pub default_color: GlyphonColor,
//     pub last_frame_touched: u64,
// }

// pub struct FullText {
//     pub buffer: GlyphonBuffer,
//     pub params: TextAreaParams,
// }

// pub struct FullTextEdit {
//     pub editor: Editor<'static>,
//     pub params: TextAreaParams,
//     pub history: TextEditHistory,
// }

// impl TextSlabs {
//     // Method to get an iterator over all buffers
//     pub fn all_text_buffers_iter(&mut self, current_frame: u64) -> impl Iterator<Item = TextArea<'_>> + '_ {
//         // Create an iterator over text box buffers
//         let text_box_buffers = self.boxes.iter_mut()
//             .map(move |text_box| if text_box.params.last_frame_touched == current_frame {
//                 Some(text_box.glyphon_text_area())
//             } else {
//                 None
//             });
        
//         // Create an iterator over text edit box buffers
//         let text_edit_box_buffers = self.editors.iter_mut()
//             .map(move |(_, editor)| if editor.params.last_frame_touched == current_frame {
//                 Some(editor.glyphon_text_area())
//             } else {
//                 None
//             });
        
//         // Chain them together
//         text_box_buffers.chain(text_edit_box_buffers).filter_map(|opt| opt)
//     }
// }

