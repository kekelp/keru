use crate::*;

/// A context for custom drawing.
/// 
/// This is a limited version of the `keru_draw` [`Renderer`] that exposes the methods for drawing, but not the internal ones like [`Renderer::clear_for_new_frame`].
pub struct Canvas<'a> {
    renderer: &'a mut Renderer,
}

impl Renderer {
    /// Get a restricted drawing context that only exposes the drawing methods, but not the methods like [`Self::begin_frame()`] and [`Self::clear_for_new_frame()`].
    pub fn get_draw_context(&mut self) -> Canvas<'_> {
        Canvas { renderer: self }
    }
}

impl<'a> Canvas<'a> {
    /// Draw a box/rectangle.
    pub fn draw_box(&mut self, params: Rectangle) {
        self.renderer.draw_box(params);
    }

    /// Draw an image.
    pub fn draw_image(&mut self, image: LoadedImage, x: f32, y: f32, width: f32, height: f32,) {
        self.renderer.draw_image(image, x, y, width, height);
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

    /// Load and rasterize an SVG, returning the loaded image.
    /// 
    /// Must be unloaded manually with [`Self::unload_image()`].
    pub fn load_svg(&mut self, svg_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        self.renderer.image_renderer.load_svg(svg_data, width, height)
    }

    /// Rerasterize an SVG at a new size if needed.
    pub fn rerasterize_svg_if_needed(&mut self, loaded: &mut LoadedImage, width: u32, height: u32) -> bool {
        self.renderer.image_renderer.rerasterize_svg_if_needed(loaded, width, height)
    }

    /// Remove a loaded SVG and free its atlas space..
    /// 
    /// Must be unloaded manually with [`Self::unload_image()`].
    pub fn unload_svg(&mut self, loaded: &LoadedImage) {
        self.renderer.image_renderer.unload_svg(loaded);
    }

    /// Load a raster image from raw RGBA8 bytes.
    /// 
    /// Returns None if the image couldn't be stored in the atlas..
    /// 
    /// Must be unloaded manually with [`Self::unload_image()`].
    pub fn load_rgba8_image(&mut self, rgba_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        self.renderer.image_renderer.load_rgba8_image(rgba_data, width, height)
    }

    /// Load a raster image from encoded bytes (PNG, JPEG, etc.).
    /// 
    /// Returns None if the image couldn't be stored in the atlas..
    /// 
    /// Must be unloaded manually with [`Self::unload_image()`].
    pub fn load_encoded_image(&mut self, image_data: &[u8]) -> Option<LoadedImage> {
        self.renderer.image_renderer.load_encoded_image(image_data)
    }

    /// Remove a loaded image and free its atlas space..
    ///
    /// Must be unloaded manually with [`Self::unload_image()`].
    pub fn unload_image(&mut self, loaded: &LoadedImage) {
        self.renderer.image_renderer.unload_image(loaded);
    }

    /// Store a gradient in the resource buffer and return a handle that can be reused
    /// across multiple draw calls via [`ColorFill::SharedGradient`].
    pub fn create_gradient(&mut self, gradient: Gradient) -> SharedGradient {
        self.renderer.create_gradient(gradient)
    }
}
