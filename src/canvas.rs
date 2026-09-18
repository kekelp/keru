use keru_draw::{
    Renderer, Rectangle, Circle, CircleRing, CircleArc, CirclePie, Segment, Grid, Triangle,
    Hexagon, Polygon, QuadraticBezier, DashedBoxOutline, DashedHexagonOutline, Transform,
    ClipRect, ClipRectHandle, LoadedImage, TextureOptions, Gradient, SharedGradient,
};
use slab::Slab;
use crate::ui::LoadedImageEntry;
use crate::render::ImageRef;
use crate::LoadedImageHandle;

/// A context for custom vector drawing inside a node, handed to the closure of [`crate::UiNode::canvas_drawing`].
///
/// A texture for the shapes is set through [`Canvas::set_texture`], which resolves a [`LoadedImageHandle`] to its atlas entry before handing it to the renderer. The raw atlas reference never leaves the `Canvas`, so it cannot outlive the handle that keeps it loaded.
pub struct Canvas<'a> {
    renderer: &'a mut Renderer,
    images: &'a Slab<LoadedImageEntry>,
}

impl<'a> Canvas<'a> {
    pub(crate) fn new(renderer: &'a mut Renderer, images: &'a Slab<LoadedImageEntry>) -> Self {
        Self { renderer, images }
    }

    /// Resolve a handle to its atlas reference, if the image is loaded and ready.
    fn resolve(&self, handle: &LoadedImageHandle) -> Option<LoadedImage> {
        self.images.get(handle.id).map(|entry| match &entry.imageref {
            ImageRef::Raster(loaded) | ImageRef::Svg(loaded) => *loaded,
        })
    }

    /// Set the texture for the draw calls that follow, until [`Self::clear_texture`]. It is sampled over each shape and multiplied by that shape's fill, so a white fill shows the texture unchanged and a coloured fill tints it. A texture that is not yet loaded resolves to none, so the shapes draw with their fill only until it is ready.
    pub fn set_texture(&mut self, texture: &LoadedImageHandle, options: Option<TextureOptions>) {
        match self.resolve(texture) {
            Some(loaded) => self.renderer.set_texture(loaded, options),
            None => self.renderer.clear_texture(),
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

    /// Set a transform for subsequent draw calls.
    pub fn set_transform(&mut self, transform: Transform) {
        let handle = self.renderer.insert_transform(transform);
        self.renderer.set_current_transform(handle);
    }

    /// Clear the current transform, resetting to identity.
    pub fn clear_transform(&mut self) {
        self.renderer.clear_current_transform();
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
