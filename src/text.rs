use parley2::StyleHandle;

use crate::*;

#[derive(Debug)]
pub enum TextI {
    TextBox(parley2::TextBoxHandle),
    StaticTextBox(parley2::StaticTextBoxHandle),
    TextEdit(parley2::TextEditHandle),
}

#[derive(Debug)]
enum DesiredTextWidget {
    TextEdit,
    TextBox, 
    StaticTextBox,
}

impl Ui {
    pub(crate) fn set_text(&mut self, i: NodeI, text: crate::NodeText, text_options: Option<&TextOptions>, style: Option<&StyleHandle>, placeholder: Option<&str>) -> &mut Self {
        // Determine what type of text widget we want
        let edit = text_options.map(|to| to.editable).unwrap_or(false);
        let selectable = text_options.map(|to| to.selectable).unwrap_or(true);
        let edit_disabled = text_options.map(|to| to.edit_disabled).unwrap_or(false);
        let single_line = text_options.map(|to| to.single_line).unwrap_or(false);
        
        let desired = if edit {
            DesiredTextWidget::TextEdit
        } else {
            match text {
                crate::NodeText::Static(_) => DesiredTextWidget::StaticTextBox,
                crate::NodeText::Dynamic(_) => DesiredTextWidget::TextBox,
            }
        };

        let needs_new_widget = match (&self.nodes[i].text_i, &desired) {
            (None, _) => true,
            (Some(TextI::TextEdit(_)), DesiredTextWidget::TextEdit) => false,
            (Some(TextI::TextBox(_)), DesiredTextWidget::TextBox) => false,
            (Some(TextI::StaticTextBox(_)), DesiredTextWidget::StaticTextBox) => false,
            _ => true, // Type mismatch, need to switch
        };

        if needs_new_widget {
            // Remove old widget
            if let Some(old_text_i) = self.nodes[i].text_i.take() {
                match old_text_i {
                    TextI::TextBox(handle) => self.sys.text.remove_text_box(handle),
                    TextI::StaticTextBox(handle) => self.sys.text.remove_static_text_box(handle),
                    TextI::TextEdit(handle) => self.sys.text.remove_text_edit(handle),
                }
            }

            // Create new widget
            let new_text_i = match desired {
                DesiredTextWidget::TextEdit => {
                    let handle = self.sys.text.add_text_edit(text.as_str().to_string(), (0.0, 0.0), (500.0, 500.0), 0.5);
                    if let Some(style) = style {
                        self.sys.text.get_text_edit_mut(&handle).set_style(style);
                    }
                    TextI::TextEdit(handle)
                },
                DesiredTextWidget::TextBox => {
                    let handle = self.sys.text.add_text_box(text.as_str().to_string(), (0.0, 0.0), (500.0, 500.0), 0.5);
                    if let Some(style) = style {
                        self.sys.text.get_text_box_mut(&handle).set_style(style);
                    }
                    TextI::TextBox(handle)
                },
                DesiredTextWidget::StaticTextBox => {
                    let handle = match text {
                        crate::NodeText::Static(s) => self.sys.text.add_static_text_box(s, (0.0, 0.0), (500.0, 500.0), 0.5),
                        crate::NodeText::Dynamic(_) => unreachable!("StaticTextBox with dynamic text"),
                    };
                    if let Some(style) = style {
                        self.sys.text.get_static_text_box_mut(&handle).set_style(style);
                    }
                    TextI::StaticTextBox(handle)
                },
            };

            self.nodes[i].text_i = Some(new_text_i);
        } else {
            // Same type - just update content and style
            match (&self.nodes[i].text_i, &desired) {
                (Some(TextI::TextEdit(handle)), DesiredTextWidget::TextEdit) => {
                    // Note: TextEdit doesn't have raw_text_mut, so we need to check if text actually changed
                    // For now, we'll just update the style if needed
                    if let Some(style) = style {
                        self.sys.text.get_text_edit_mut(&handle).set_style(style);
                    }

                    // don't update the content. content in a text edit box is not reset declaratively every frame, obviously. 
                },
                (Some(TextI::TextBox(handle)), DesiredTextWidget::TextBox) => {
                    *self.sys.text.get_text_box_mut(&handle).raw_text_mut() = text.as_str().to_string();
                    if let Some(style) = style {
                        self.sys.text.get_text_box_mut(&handle).set_style(style);
                    }
                },
                (Some(TextI::StaticTextBox(handle)), DesiredTextWidget::StaticTextBox) => {
                    match text {
                        crate::NodeText::Dynamic(_) => unreachable!("Surely it's static only here"),
                        crate::NodeText::Static(s) => {
                            *self.sys.text.get_static_text_box_mut(&handle).raw_text_mut() = s;
                        },
                    };
                    

                    if let Some(style) = style {
                        self.sys.text.get_static_text_box_mut(&handle).set_style(style);
                    }
                },
                _ => unreachable!("Type mismatch should have been handled above"),
            }
        }

        // Apply text options
        if let Some(text_i) = &self.nodes[i].text_i {
            match text_i {
                TextI::TextEdit(handle) => {
                    self.sys.text.get_text_edit_mut(handle).set_disabled(edit_disabled);
                    self.sys.text.get_text_edit_mut(handle).set_single_line(single_line);
                    if let Some(placeholder) = placeholder {
                        self.sys.text.get_text_edit_mut(handle).set_placeholder(placeholder.to_string());
                    }
                },
                TextI::TextBox(handle) => {
                    self.sys.text.get_text_box_mut(handle).set_selectable(selectable);
                },
                TextI::StaticTextBox(handle) => {
                    self.sys.text.get_static_text_box_mut(handle).set_selectable(selectable);
                },
            }
        }

        self.push_text_change(i);
        self
    }

    /// Insert a style, and get a [`StyleHandle`] that can be used to access and mutate it with the [`Self::get_style_mut`] functions.
    /// 
    /// This function **should not be called on every frame**, as that would insert a new copy of the style every time.
    /// 
    // todo: figure out a better way to do this.  
    pub fn insert_style(&mut self, style: TextStyle) -> StyleHandle {
        self.sys.text.add_style(style, None)
    }

    pub fn get_style(&self, style: &StyleHandle) -> &TextStyle {
        self.sys.text.get_text_style(style)
    }

    pub fn get_style_mut(&mut self, style: &StyleHandle) -> &mut TextStyle {
        self.sys.text.get_text_style_mut(style)
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

