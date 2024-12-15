use std::fmt::Display;

use glyphon::cosmic_text::Align;
use glyphon::Attrs;
use glyphon::Family;
use glyphon::Shaping;

use crate::*;
use crate::node::*;
use crate::Axis::*;

/// A struct referring to a node in the GUI tree.
/// 
/// A `UiNode` is returned when "added" the node to the tree through [`Ui::add`] or similar functions. In that case, you can use the [`UiNode`]'s builder methods to set the node's params, size, color, text, image, etc., and to eventually [place](`UiNode::place`) the node onto the tree.
/// 
/// A `UiNode` is also returned from [`Ui::get_node`]. This is useful to extract dynamic properties of a node, like its exact size.
pub struct UiNode<'a> {
    pub(crate) node_i: usize,
    pub(crate) ui: &'a mut Ui,
}

impl<'a> UiNode<'a> {
    pub(crate) fn node_mut(&mut self) -> &mut Node {
        return &mut self.ui.nodes.nodes[self.node_i];
    }
    pub(crate) fn node(&self) -> &Node {
        return &self.ui.nodes.nodes[self.node_i];
    }

    /// Add an image to the node.
    /// 
    /// Uses pointer comparison to tell if the image has changed since the last call on the same node..
    /// 
    /// ```rust
    /// # use keru::*;
    /// # pub struct State {
    /// #     pub ui: Ui,
    /// # }
    /// #
    /// # impl State {
    /// #    fn declare_ui(&mut self) {
    /// #    let ui = &mut self.ui; 
    /// #
    /// # #[node_key] pub const MY_BUTTON: NodeKey;
    /// #
    /// ui.add(MY_BUTTON).params(ICON_BUTTON).static_image(include_bytes!("textures/debug.png"));
    /// #
    /// #   }
    /// # }
    /// ```
    /// 
    /// Panics if the byte slice in `image` can't be interpreted as an image.
    pub fn static_image(&mut self, image: &'static [u8]) {
        let image_pointer: *const u8 = image.as_ptr();

        if let Some(last_pointer) = self.node().last_static_image_ptr {
            if image_pointer == last_pointer {
                return;
            }
        }

        let image = self.ui.sys.texture_atlas.allocate_image(image);
        self.node_mut().imageref = Some(image);
        self.node_mut().last_static_image_ptr = Some(image_pointer);
    }

    /// Add an image to the node.
    /// 
    /// If `changed` is `false`, it will assume that the same image as the last frame is being passed, and won't do anything.
    /// 
    /// Otherwise, it will assume that it has changed.
    /// 
    /// Panics if the byte slice in `image` can't be interpreted as an image.
    pub fn dyn_image(&mut self, image: &[u8], changed: bool) {
        if self.node_mut().imageref.is_some() && changed == false {
            return;
        }

        let image = self.ui.sys.texture_atlas.allocate_image(image);
        self.node_mut().imageref = Some(image);
        self.node_mut().last_static_image_ptr = None;
    }

    // This is not a callback, the effect is executed immediately (or never if not clicked)
    // It's this way just for easier builder-style composition
    // You can also do ui.is_clicked(KEY) 
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

    pub fn color(&mut self, color: crate::color::Color) -> &mut Self {
        self.node_mut().params.rect.vertex_colors = VertexColors::flat(color);
        return self;
    }

    pub fn vertex_colors(&mut self, colors: VertexColors) -> &mut Self {
        self.node_mut().params.rect.vertex_colors = colors;
        return self;
    }

    pub fn position(&mut self, position_x: Position, position_y: Position) -> &mut Self {
        self.node_mut().params.layout.position.x = position_x;
        self.node_mut().params.layout.position.y = position_y;
        return self;
    }

    pub fn position_symm(&mut self, position: Position) -> &mut Self {
        self.node_mut().params.layout.position.x = position;
        self.node_mut().params.layout.position.y = position;
        return self;
    }

    pub fn position_x(&mut self, position: Position) -> &mut Self {
        self.node_mut().params.layout.position.x = position;
        return self;
    }

    pub fn position_y(&mut self, position: Position) -> &mut Self {
        self.node_mut().params.layout.position.y = position;
        return self;
    }

    pub fn size(&mut self, size_x: Size, size_y: Size) -> &mut Self {
        self.node_mut().params.layout.size.x = size_x;
        self.node_mut().params.layout.size.y = size_y;
        return self;
    }

    pub fn size_symm(&mut self, size: Size) -> &mut Self {
        self.node_mut().params.layout.size.x = size;
        self.node_mut().params.layout.size.y = size;
        return self;
    }
    
    pub fn size_x(&mut self, size: Size) -> &mut Self {
        self.node_mut().params.layout.size.x = size;
        return self;
    }

    pub fn size_y(&mut self, size: Size) -> &mut Self {
        self.node_mut().params.layout.size.y = size;
        return self;
    }

    pub fn params(&mut self, params: NodeParams) -> &mut Self {
        self.node_mut().params = params;
        return self;
    }

    pub fn inner_size(&self) -> Xy<u32> {
        let padding = self.node().params.layout.padding;
        let padding = self.ui.to_pixels2(padding);
        
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

        let center = center * self.ui.sys.part.unifs.size;

        return center;
    }

    pub fn bottom_left(&self) -> Xy<f32> {
        let rect = self.node().rect;
        
        let center = Xy::new(
            rect[X][0],
            rect[Y][1],
        );

        let center = center * self.ui.sys.part.unifs.size;
        
        return center;
    }

    pub fn rect(&self) -> XyRect {
        return self.node().rect * self.ui.sys.part.unifs.size;
    }

    pub fn render_rect(&self) -> RenderInfo {
        return RenderInfo {
            rect: self.node().rect.to_graphics_space(),
            z: self.node().z + Z_STEP / 2.0,
        };
    }

    pub fn stack_arrange(&mut self, arrange: Arrange) -> &mut Self {
        let stack = match self.node().params.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.node_mut().params.stack = Some(stack.arrange(arrange));
        return self;
    }

    pub fn stack_spacing(&mut self, spacing: Len) -> &mut Self {
        let stack = match self.node().params.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.node_mut().params.stack = Some(stack.spacing(spacing));
        return self;
    }

    pub fn padding(&mut self, padding: Len) -> &mut Self {
        self.node_mut().params.layout.padding = Xy::new_symm(padding);
        return self;
    }

    pub fn padding_x(&mut self, padding: Len) -> &mut Self {
        self.node_mut().params.layout.padding.x = padding;
        return self;
    }

    pub fn padding_y(&mut self, padding: Len) -> &mut Self {
        self.node_mut().params.layout.padding.y = padding;
        return self;
    }

    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.node_mut().params.rect.shape = shape;
        return self;
    }
}

impl<'a> UiNode<'a> {
    
    /// Add some text to the node.
    /// 
    /// Uses pointer equality to check if the text has changed since the last call on the same node.
    /// 
    /// ```rust
    /// # use keru::*;
    /// # pub struct State {
    /// #     pub ui: Ui,
    /// #     pub show: bool,
    /// # }
    /// #
    /// # impl State {
    /// #    fn declare_ui(&mut self) {
    /// #    let ui = &mut self.ui; 
    /// #
    /// # #[node_key] pub const SHOW: NodeKey;
    /// let button_text = match self.show {
    ///     true => "Hide Counter",
    ///     false => "Show Counter",
    /// };
    /// ui.add(SHOW)
    ///     .params(BUTTON)
    ///     .static_text(button_text);
    /// #
    /// #   }
    /// # }
    /// ```
    pub fn static_text(&mut self, text: &'static str) -> &mut Self {
        let text_pointer: *const u8 = text.as_ptr();

        if let Some(last_pointer) = self.node().last_static_text_ptr {
            if text_pointer == last_pointer {
                return self;
            }
        }

        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_unchecked(text_id, text);
        } else {
            let text_id = self
                .ui
                .sys
                .text
                .maybe_new_text_area(Some(text), self.ui.sys.part.current_frame);
            self.node_mut().text_id = text_id;
        }

        self.node_mut().last_static_text_ptr = Some(text_pointer);

        self.ui.push_text_change(self.node_i);

        return self;
    }

    /// Add some text to the node.
    /// 
    /// Will hash the provided text to determine if has changed since the last call on the same node.
    /// 
    /// ```rust
    /// # use keru::*;
    /// # pub struct State {
    /// #     pub ui: Ui,
    /// # }
    /// #
    /// # impl State {
    /// #    fn declare_ui(&mut self) {
    /// #    let ui = &mut self.ui; 
    /// #
    /// # #[node_key] pub const MY_BUTTON: NodeKey;
    /// #
    /// # use std::time::Instant;
    /// let variable_text = format!("{:?}", Instant::now());
    /// ui.add(MY_BUTTON)
    ///     .params(BUTTON)
    ///     .text(variable_text);
    /// #
    /// #   }
    /// # }
    /// ```

    pub fn text(&mut self, into_text: impl Display) -> &mut Self {
        // todo: hash into_text instead of the text to skip the formatting??
        // note that many exotic types like "f32" can be formatted but not hashed 
        self.ui.format_into_scratch(into_text);

        if let Some(text_id) = self.node_mut().text_id {
            let hash = fx_hash(&self.ui.format_scratch);
            let area = &mut self.ui.sys.text.text_areas[text_id];
            if hash != area.params.last_hash {
                area.params.last_hash = hash;
                area.buffer.set_text(
                    &mut self.ui.sys.text.font_system,
                    &self.ui.format_scratch,
                    Attrs::new().family(Family::SansSerif),
                    Shaping::Advanced,
                );

                self.ui.push_text_change(self.node_i);
            }
        } else {
            let text_id = self
                .ui
                .sys
                .text
                .maybe_new_text_area(Some(&self.ui.format_scratch), self.ui.sys.part.current_frame);
            self.node_mut().text_id = text_id;
            self.ui.push_text_change(self.node_i);
        }

        return self;
    }

    /// Add some text to the node.
    /// 
    /// If `into_text` is `None`, the function will assume that the text hasn't changed since the last call, and won't do anything.
    /// 
    /// Otherwise, it will assume that it has changed.
    pub fn dyn_text(mut self, into_text: Option<impl Display>) -> Self {
        // if the text is None, return.
        let Some(into_text) = into_text else {
            return self;
        };
        
        self.ui.format_into_scratch(into_text);
        
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_unchecked(text_id, &self.ui.format_scratch);
        } else {
            let text_id = self
                .ui
                .sys
                .text
                .maybe_new_text_area(Some(&self.ui.format_scratch), self.ui.sys.part.current_frame);
            self.node_mut().text_id = text_id;
        }

        self.ui.push_text_change(self.node_i);

        return self;
    }

    /// Set the node's text attrs to `attrs`.
    /// 
    /// `attrs` is a `cosmic_text::Attrs` object. 
    pub fn text_attrs(&mut self, attrs: Attrs) -> &mut Self {
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_attrs(text_id, attrs);

            self.ui.set_partial_relayout_flag(self.node_i);

        } else {
            // todo: add the text area
        }
        return self;
    }

    /// Set the node's text align to `align`.
    /// 
    /// `align` is a `cosmic_text::Align` object. 
    pub fn text_align(&mut self, align: Align) -> &mut Self {
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_align(text_id, align);
        } else {
            // todo: add the text area
        }
        return self;
    }

    // todo: in a sane world, this wouldn't allocate.
    pub fn get_text(&self) -> Option<String> {
        // let text_id = self.node().text_id.unwrap();

        // let lines = self.ui.sys.text.text_areas[text_id].buffer.lines;
        
        // let text = lines.into_iter().map(|l| l.text()).collect();
        // return Some(text);
        return None;
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