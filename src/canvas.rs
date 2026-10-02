use keru_draw::{
    Renderer, Rectangle, Circle, CircleRing, CircleArc, CirclePie, Segment, Grid, Triangle,
    Hexagon, Polygon, QuadraticBezier, DashedBoxOutline, DashedHexagonOutline,
    ClipRect, ClipRectHandle, LoadedImage, TextureOptions, Gradient, SharedGradient,
    OkhsvSaturationValueGradient, OkhsvHueGradient,
};
use crate::ui::{ImageStore, LoadedImageId, load_path_image};
use crate::inner_node::ImageSourceId;
use crate::ahash;
use crate::render::ImageRef;
use crate::LoadedImageHandle;

/// A context for custom vector drawing inside a node, handed to the closure of [`crate::UiNode::canvas_drawing`].
pub struct Canvas<'a> {
    renderer: &'a mut Renderer,
    images: &'a mut ImageStore,
    // The node will have to keep track of the images used and clear them from the atlas when it disappears.
    canvas_images: &'a mut Vec<LoadedImageId>,
}

impl<'a> Canvas<'a> {
    pub(crate) fn new(
        renderer: &'a mut Renderer,
        images: &'a mut ImageStore,
        canvas_images: &'a mut Vec<LoadedImageId>,
        ) -> Self {
        Self { renderer, images, canvas_images }
    }

    /// Resolve a handle to its atlas reference, if the image is loaded and ready.
    fn resolve(&self, handle: &LoadedImageHandle) -> Option<LoadedImage> {
        self.images.get(handle.id).map(|entry| match &entry.imageref {
            ImageRef::Raster(loaded) | ImageRef::Svg(loaded) => *loaded,
        })
    }

    /// Set the texture for the draw calls that follow, until [`Self::clear_texture`].
    pub fn set_texture(&mut self, texture: &LoadedImageHandle, options: Option<TextureOptions>) {
        match self.resolve(texture) {
            Some(loaded) => self.renderer.set_texture(loaded, options),
            None => self.renderer.clear_texture(),
        }
    }

    /// Set the texture for the draw calls that follow using an image file path, until [`Self::clear_texture`].
    pub fn set_texture_from_path(&mut self, path: &str, options: Option<TextureOptions>) {
        let key = (ImageSourceId::PathHash(ahash(&path)), None);
        let id = match self.images.get_new_ref_from_cache(&key) {
            Some(id) => id,
            None => {
                let Some(imageref) = load_path_image(self.renderer, path) else {
                    self.renderer.clear_texture();
                    return;
                };
                self.images.save_in_cache(key, imageref)
            }
        };
        self.canvas_images.push(id);
        match self.images[id].imageref {
            ImageRef::Raster(loaded) | ImageRef::Svg(loaded) => self.renderer.set_texture(loaded, options),
        }
    }

    /// Clear the current texture, so subsequent draws use only their own fill.
    pub fn clear_texture(&mut self) {
        self.renderer.clear_texture();
    }

    /// Draw a box/rectangle.
    pub fn draw_box(&mut self, params: Rectangle) {
        self.renderer.draw_box(params);
    }

    /// Draw an image at the given position and size, from a loaded-image handle.
    pub fn draw_image(&mut self, image: &LoadedImageHandle, x: f32, y: f32, width: f32, height: f32) {
        if let Some(loaded) = self.resolve(image) {
            self.renderer.draw_image(loaded, x, y, width, height);
        }
    }

    /// Draw a filled circle.
    pub fn draw_circle(&mut self, params: Circle) {
        self.renderer.draw_circle(params);
    }

    /// Draw a ring (hollow circle).
    pub fn draw_ring(&mut self, params: CircleRing) {
        self.renderer.draw_ring(params);
    }

    /// Draw a fixed-hue OKHSV saturation/value color picker square.
    pub fn draw_okhsv_saturation_value_gradient(&mut self, params: OkhsvSaturationValueGradient) {
        self.renderer.draw_okhsv_saturation_value_gradient(params);
    }

    /// Draw a full-hue OKHSV selection strip at a fixed saturation/value.
    pub fn draw_okhsv_hue_gradient(&mut self, params: OkhsvHueGradient) {
        self.renderer.draw_okhsv_hue_gradient(params);
    }

    /// Draw an arc.
    pub fn draw_arc(&mut self, params: CircleArc) {
        self.renderer.draw_arc(params);
    }

    /// Draw a pie slice.
    pub fn draw_pie(&mut self, params: CirclePie) {
        self.renderer.draw_pie(params);
    }

    /// Draw a line segment.
    pub fn draw_segment(&mut self, params: Segment) {
        self.renderer.draw_segment(params);
    }

    /// Draw a grid.
    pub fn draw_grid(&mut self, params: Grid) {
        self.renderer.draw_grid(params);
    }

    /// Draw a triangle.
    pub fn draw_triangle(&mut self, params: Triangle) {
        self.renderer.draw_triangle(params);
    }

    /// Draw a hexagon.
    pub fn draw_hexagon(&mut self, params: Hexagon) {
        self.renderer.draw_hexagon(params);
    }

    /// Draw a filled or stroked arbitrary polygon.
    pub fn draw_polygon(&mut self, params: Polygon) {
        self.renderer.draw_polygon(params);
    }

    /// Draw a quadratic bezier curve.
    pub fn draw_quadratic_bezier(&mut self, params: QuadraticBezier) {
        self.renderer.draw_quadratic_bezier(params);
    }

    /// Draw a dashed box outline.
    pub fn draw_dashed_box_outline(&mut self, params: DashedBoxOutline) {
        self.renderer.draw_dashed_box_outline(params);
    }

    /// Draw a dashed hexagon outline.
    pub fn draw_dashed_hexagon_outline(&mut self, params: DashedHexagonOutline) {
        self.renderer.draw_dashed_hexagon_outline(params);
    }

    /// Create a clip rect for this frame.
    pub fn insert_clip_rect(&mut self, clip_rect: ClipRect) -> ClipRectHandle {
        self.renderer.insert_clip_rect(clip_rect)
    }

    /// Set a clip rect for subsequent draw calls.
    pub fn set_clip_rect(&mut self, clip_rect: ClipRect) {
        let handle = self.renderer.insert_clip_rect(clip_rect);
        self.renderer.set_current_clip_rect(handle);
    }

    /// Clear the current clip rect, resetting to no clip.
    pub fn clear_clip_rect(&mut self) {
        self.renderer.clear_current_clip_rect();
    }

    /// Store a gradient and return a handle reusable across draw calls via [`keru_draw::ColorFill::SharedGradient`].
    pub fn create_gradient(&mut self, gradient: Gradient) -> SharedGradient {
        self.renderer.create_gradient(gradient)
    }
}
