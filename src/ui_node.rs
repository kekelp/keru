use glyphon::Attrs;
use glyphon::Family;
use glyphon::Shaping;

use crate::*;
use crate::node::*;
use crate::Axis::*;

/// A struct referring to a node in the GUI tree.
/// 
/// A `UiNode` is returned from [`Ui::get_node`]. This is useful to extract dynamic properties of a node, like its exact size.
pub struct UiNode<'a> {
    pub(crate) i: NodeI,
    pub(crate) ui: &'a mut Ui,
}

// todo: clean up all the setter functions here and move them to nodeparams.
impl<'a> UiNode<'a> {
    pub(crate) fn node_mut(&mut self) -> &mut Node {
        return &mut self.ui.nodes[self.i];
    }
    pub(crate) fn node(&self) -> &Node {
        return &self.ui.nodes[self.i];
    }

    pub(crate) fn static_image(&mut self, image: &'static [u8]) -> &mut Self {
        let image_pointer: *const u8 = image.as_ptr();

        if let Some(last_pointer) = self.node().last_static_image_ptr {
            if image_pointer == last_pointer {
                return self;
            }
        }

        let image = self.ui.sys.texture_atlas.allocate_image(image);
        self.node_mut().imageref = Some(image);
        self.node_mut().last_static_image_ptr = Some(image_pointer);

        return self;
    }

    // This is not a callback, the effect is executed immediately (or never if not clicked)
    // It's this way just for easier builder-style composition
    // You can also do ui.is_clicked(KEY) 
    // #[must_use]
    // pub fn on_click(&mut self, effect: impl FnOnce()) -> &mut Self {
    //     let id = self.node().id;

    //     let is_clicked = self.ui
    //     .sys
    //     .last_frame_clicks
    //     .clicks
    //     .iter()
    //     .any(|c| c.hit_node_id == id && c.state.is_pressed() && c.button == MouseButton::Left); 

    //     if is_clicked {
    //         effect();
    //     }

    //     return self;
    // }

    // pub fn is_dragged(&self) -> Option<(f64, f64)> {
    //     if self.is_clicked(node_key) {
    //         return Some(self.sys.mouse_status.cursor_diff())
    //     } else {
    //         return None;
    //     }
    // }

    pub fn inner_size(&self) -> Xy<u32> {
        let padding = self.node().params.layout.padding;
        
        let size = self.node().size;
        let size = self.ui.f32_size_to_pixels2(size);

        return size - padding;
    }

    pub fn center(&self) -> Xy<f32> {
        let rect = self.node().rect;
        
        let center = Xy::new(
            (rect[X][1] + rect[X][0]) / 2.0,
            (rect[Y][1] + rect[Y][0]) / 2.0,
        );

        let center = center * self.ui.sys.unifs.size;

        return center;
    }

    pub fn bottom_left(&self) -> Xy<f32> {
        let rect = self.node().rect;
        
        let center = Xy::new(
            rect[X][0],
            rect[Y][1],
        );

        let center = center * self.ui.sys.unifs.size;
        
        return center;
    }

    pub fn rect(&self) -> XyRect {
        return self.node().rect * self.ui.sys.unifs.size;
    }

    pub fn render_rect(&self) -> RenderInfo {
        return RenderInfo {
            rect: self.node().rect.to_graphics_space(),
            z: self.node().z + Z_STEP / 2.0,
        };
    }
}

impl<'a> UiNode<'a> {
    // todo: move to Ui, or just merge with set_params_text, or something
    pub(crate) fn text_from_fmtscratch(&mut self) -> &mut Self {
        // assume that the caller wrote the text into format_scratch...

        if let Some(text_id) = self.node_mut().text_id {
            let area = &mut self.ui.sys.text.text_areas[text_id];
            area.buffer.set_text(
                &mut self.ui.sys.text.font_system,
                &self.ui.format_scratch,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );

            self.ui.push_text_change(self.i);
        
        } else {
            let text_id = self
                .ui
                .sys
                .text
                .maybe_new_text_area(Some(&self.ui.format_scratch), self.ui.sys.current_frame);
            self.node_mut().text_id = text_id;
            self.ui.push_text_change(self.i);
        }

        return self;
    }

    // todo: in a sane world, this wouldn't allocate.
    pub fn get_text(&self) -> Option<String> {
        let text_id = self.node().text_id?;

        let lines = &self.ui.sys.text.text_areas[text_id].buffer.lines;
        
        let text = lines.into_iter().map(|l| l.text()).collect();
        return Some(text);
    }
}

/// The data needed for rendering a node with custom code.
/// 
/// Obtained from a [`UiNode`] through [`UiNode::render_rect`]
/// 
/// The data is ready to be used in a shader like this:
/// 
/// ```wgsl
/// struct Rect {
///     @location(0) xs: vec2<f32>,
///     @location(1) ys: vec2<f32>,
///     @location(2) z: f32,
/// }
/// ```
/// 
/// With these vertex attributes:
/// 
/// ```rust
/// # use keru::*;
/// wgpu::vertex_attr_array![
///     0 => Float32x2,
///     1 => Float32x2,
///     2 => Float32,
/// ];
/// ```
/// 
/// The format might be changed to something more familiar in the future.
/// 
/// This doesn't include the information about the `Shape`, because it's harder to interpret, and it's usually static.
#[derive(Copy, Clone, Debug)]
pub struct RenderInfo {
    pub rect: XyRect,
    pub z: f32,
}