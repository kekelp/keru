use std::{marker::PhantomData, mem};

use bytemuck::Pod;
use glam::dvec2;
use vello_common::peniko::Gradient;
use wgpu::{Buffer, BufferSlice, Device, Queue};
use winit::event::*;
use winit::window::Window;

use vello_common::{kurbo::{Rect as VelloRect, RoundedRect, Circle, BezPath}, paint::PaintType};
use peniko::color::AlphaColor;
use vello_common::kurbo::Shape as VelloShape;

use crate::*;

impl Ui {
    /// Handles window events and updates the `Ui`'s internal state accordingly.
    /// 
    /// You can then check for input on specific nodes with [`Ui::is_clicked`] and similar functions.
    ///
    /// You should pass all events from winit to this method, unless they are "consumed" by something "above" the GUI.
    ///
    /// Returns `true` if the event was "consumed" by the `Ui`, e.g. if a mouse click hit an opaque panel.
    /// 
    // todo: move or rename the file
    pub fn window_event(&mut self, event: &WindowEvent, window: &Window) -> bool {
        self.sys.mouse_input.window_event(event);
        self.sys.key_input.window_event(event);

        self.text_window_event(ROOT_I, event, window);

        self.ui_input(&event, window);
        
        return false;
    }

    fn text_window_event(&mut self, _i: NodeI, event: &WindowEvent, window: &Window){     
        self.sys.text.handle_event(event, window);
        
        let text_changed = self.sys.text.any_text_changed();
        if text_changed {
            if let Some(node_id) = self.sys.focused {
                self.sys.text_edit_changed_this_frame = Some(node_id);
            }
            self.sys.changes.text_changed = true;
        }
    }

    pub fn ui_input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        match event {
            WindowEvent::RedrawRequested => {
                self.new_redraw_requested_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {
                // Set new input if this is a hover or a drag
                let last_cursor_pos = self.sys.mouse_input.prev_cursor_position();
                if dvec2(position.x, position.y) != last_cursor_pos {
                    let has_hover_sense = self.sys.hovered.iter()
                        .filter_map(|id| self.nodes.get_by_id_ick(id).map(|(node, _)| node))
                        .any(|node| node.params.interact.senses.contains(Sense::HOVER));

                    let has_drag = self.sys.mouse_input.currently_pressed_tags()
                        .filter_map(|(tag, _)| tag.and_then(|id| self.nodes.get_by_id_ick(&id).map(|(node, _)| node)))
                        .any(|node| node.params.interact.senses.contains(Sense::DRAG));

                    if has_hover_sense || has_drag {
                        self.set_new_ui_input();
                    }
                }

                self.resolve_hover();
                // cursormoved is never consumed
            }
            WindowEvent::MouseInput { button, state, .. } => {

                let clicked_id = self.sys.mouse_input.current_tag();
                let Some(clicked_i) = self.nodes.get_by_id(clicked_id) else { return false };

                match state {
                    ElementState::Pressed => {
                        let consumed = self.resolve_click_press(*button, event, window, clicked_i);
                        return consumed;
                    },
                    ElementState::Released => {
                        self.resolve_click_release(*button, clicked_i);
                        // Consuming mouse releases can very easily mess things up for whoever is below us.
                        // Some unexpected mouse releases probably won't be too annoying.
                        return false
                    },
                }
            }
            WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                // todo: set new_input only if a node is focused? hard to tell... users probably *shouldn't* listen for unconditional key inputs, but they definitely can
                // probably should have two different bools: one for focused input, one for generic new input. the event loop can decide to wake up and/or update depending on either 
                self.set_new_ui_input();
                if !is_synthetic {
                    let consumed = self.handle_keyboard_event(event);
                    return consumed;
                }
            }
            WindowEvent::Ime(_) => {
                self.set_new_ui_input();
            }
            WindowEvent::Moved(..) => {
                self.resolve_hover();
            }
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_scroll_event(delta);
                self.set_new_ui_input();
            }
            _ => {}
        }
        return false;
    }


    /// Render a node's shape directly to the vello_hybrid scene.
    pub(crate) fn render_node_shape_to_scene(&mut self, i: NodeI) {
        let node = &self.nodes[i];

        // Get rect in normalized space (0-1)
        let animated_rect = node.get_animated_rect();

        // Convert to pixel coordinates
        let screen_size = self.sys.unifs.size;
        let x0 = (animated_rect.x[0] * screen_size.x).round() as f64;
        let y0 = (animated_rect.y[0] * screen_size.y).round() as f64;
        let x1 = (animated_rect.x[1] * screen_size.x).round() as f64;
        let y1 = (animated_rect.y[1] * screen_size.y).round() as f64;

        // Get vertex colors
        let colors = node.params.rect.vertex_colors;
        let tl = colors.top_left_color();
        let tr = colors.top_right_color();
        let bl = colors.bottom_left_color();
        let br = colors.bottom_right_color();

        let is_solid = tl == tr && tl == bl && tl == br;
        
        let tl_alpha = AlphaColor::from_rgba8(tl.r, tl.g, tl.b, tl.a);
        let br_alpha = AlphaColor::from_rgba8(br.r, br.g, br.b, br.a);

        if is_solid {
            self.sys.vello_scene.set_paint(PaintType::Solid(tl_alpha));
        } else {
            let gradient = Gradient::new_linear((x0, y1), (x1, y0))
                .with_stops([tl_alpha, br_alpha]);
            self.sys.vello_scene.set_paint(PaintType::Gradient(gradient));
        }

        // Render based on shape type
        match &node.params.rect.shape {
            Shape::Rectangle { corner_radius } => {
                let corner_radius = *corner_radius as f64;

                // Get which corners should be rounded
                let rounded_corners = node.params.rect.rounded_corners;
                let top_right = rounded_corners.contains(RoundedCorners::TOP_RIGHT);
                let top_left = rounded_corners.contains(RoundedCorners::TOP_LEFT);
                let bottom_right = rounded_corners.contains(RoundedCorners::BOTTOM_RIGHT);
                let bottom_left = rounded_corners.contains(RoundedCorners::BOTTOM_LEFT);

                if corner_radius > 0.0 && (top_right || top_left || bottom_right || bottom_left) {
                    // Create rounded rect with per-corner radii
                    let radii = vello_common::kurbo::RoundedRectRadii {
                        top_left: if top_left { corner_radius } else { 0.0 },
                        top_right: if top_right { corner_radius } else { 0.0 },
                        bottom_right: if bottom_right { corner_radius } else { 0.0 },
                        bottom_left: if bottom_left { corner_radius } else { 0.0 },
                    };
                    let rounded_rect = RoundedRect::from_rect(
                        VelloRect::new(x0, y0, x1, y1),
                        radii
                    );
                    self.sys.vello_scene.fill_path(&rounded_rect.to_path(0.1));
                } else {
                    let rect = VelloRect::new(x0, y0, x1, y1);
                    self.sys.vello_scene.fill_rect(&rect);
                }
            }
            Shape::Circle => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                let circle = Circle::new((cx, cy), radius);
                self.sys.vello_scene.fill_path(&circle.to_path(0.1));
            }
            Shape::Ring { width } => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let outer_radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                let inner_radius = (outer_radius - *width as f64).max(0.0);

                // Create ring by subtracting inner circle from outer circle
                let mut path = BezPath::new();
                let outer_circle = Circle::new((cx, cy), outer_radius);
                path.extend(outer_circle.to_path(0.1).iter());

                if inner_radius > 0.0 {
                    // Add inner circle in reverse to create a hole
                    let inner_circle = Circle::new((cx, cy), inner_radius);
                    let inner_path = inner_circle.to_path(0.1);
                    // Reverse the inner path to create a cutout
                    let mut reversed_inner = BezPath::new();
                    for segment in inner_path.iter().collect::<Vec<_>>().into_iter().rev() {
                        match segment {
                            vello_common::kurbo::PathEl::MoveTo(p) => reversed_inner.move_to(p),
                            vello_common::kurbo::PathEl::LineTo(p) => reversed_inner.line_to(p),
                            vello_common::kurbo::PathEl::QuadTo(p1, p2) => reversed_inner.quad_to(p1, p2),
                            vello_common::kurbo::PathEl::CurveTo(p1, p2, p3) => reversed_inner.curve_to(p1, p2, p3),
                            vello_common::kurbo::PathEl::ClosePath => reversed_inner.close_path(),
                        }
                    }
                    path.extend(reversed_inner.iter());
                }

                self.sys.vello_scene.fill_path(&path);
            }
        }
    }

    /// Load the GUI render data that has changed onto the GPU.
    fn prepare(&mut self, _device: &Device, queue: &Queue) {
        // update time + resolution. since we have to update the time anyway, we also update the screen resolution all the time
        // self.sys.unifs.t = ui_time_f32();
        // queue.write_buffer(
        //     &self.sys.base_uniform_buffer,
        //     0,
        //     bytemuck::bytes_of(&self.sys.unifs),
        // );

        // // update rects
        // if self.sys.changes.need_gpu_rect_update || self.sys.changes.should_rebuild_render_data {
        //     self.sys.gpu_rect_buffer.queue_write(&self.sys.rects[..], queue);
        //     self.sys.changes.need_gpu_rect_update = false;
        //     log::trace!("Update GPU rectangles");
        // }
        
        // texture atlas
        // todo: don't do this all the time
        // self.sys.texture_atlas.load_to_gpu(queue);
    }

    /// Renders the UI to a surface using vello_hybrid.
    ///
    /// The scene is built during `rebuild_render_data`, and this method just renders it.
    pub fn render(
        &mut self,
        surface: &wgpu::Surface,
        _depth_texture: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        if self.sys.changes.should_rebuild_render_data {
            self.rebuild_render_data();
        }

        log::trace!("Render");
        self.do_cosmetic_rect_updates();

        self.prepare(device, queue);

        self.sys.text.clear_changes();

        // Render the scene to the surface
        let surface_texture = surface.get_current_texture().unwrap();
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // Upload pending images
        let pending_images = std::mem::take(&mut self.sys.pending_images);
        for (node_i, image_bytes) in pending_images {
            use vello_common::pixmap::Pixmap;
            use vello_common::peniko::color::PremulRgba8;

            log::info!("Uploading image for node {:?}, size {} bytes", node_i, image_bytes.len());

            // Load and decode the image
            let img = image::load_from_memory(image_bytes).unwrap();
            let img = img.to_rgba8();
            let (width, height) = img.dimensions();

            log::info!("Decoded image: {}x{}", width, height);

            // Convert to premultiplied RGBA8
            let pixels: Vec<PremulRgba8> = img.pixels().map(|p| {
                let r = p[0];
                let g = p[1];
                let b = p[2];
                let a = p[3];

                let alpha = a as u16;
                let premul_r = ((r as u16 * alpha) / 255) as u8;
                let premul_g = ((g as u16 * alpha) / 255) as u8;
                let premul_b = ((b as u16 * alpha) / 255) as u8;

                PremulRgba8 { r: premul_r, g: premul_g, b: premul_b, a }
            }).collect();

            let pixmap = Pixmap::from_parts(pixels, width as u16, height as u16);

            // Upload to vello_hybrid
            let image_id = self.sys.vello_renderer.upload_image(device, queue, &mut encoder, &pixmap);

            log::info!("Uploaded image, got ImageId: {:?}", image_id);

            // Store the ImageId in the node
            self.nodes[node_i].imageref = Some(crate::texture_atlas::ImageRef {
                image_id,
                original_size: crate::math::Xy::new(width as f32, height as f32),
            });
        }

        let render_size = vello_hybrid::RenderSize {
            width: self.sys.unifs.size[Axis::X] as u32,
            height: self.sys.unifs.size[Axis::Y] as u32,
        };

        self.sys.vello_renderer.render(&self.sys.vello_scene, device, queue, &mut encoder, &render_size, &view).ok();

        queue.submit([encoder.finish()]);
        surface_texture.present();

        self.sys.changes.need_rerender = false;
    }

    /// Returns `true` if the `Ui` needs to be rerendered.
    /// 
    /// If this is true, you should call [`Ui::render`] as soon as possible to display the updated GUI state on the screen.
    pub fn should_rerender(&mut self) -> bool {
        return self.sys.changes.need_rerender
            || self.sys.anim_render_timer.is_live()
            || self.sys.text.needs_rerender()
            || self.sys.changes.should_rebuild_render_data;
    }
}



#[derive(Debug)]
pub struct TypedGpuBuffer<T: Pod> {
    pub buffer: Buffer,
    pub marker: std::marker::PhantomData<T>,
}
impl<T: Pod> TypedGpuBuffer<T> {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            marker: PhantomData::<T>,
        }
    }

    #[allow(dead_code)]
    pub fn size() -> u64 {
        mem::size_of::<T>() as u64
    }

    pub fn slice<N: Into<u64>>(&self, n: N) -> BufferSlice {
        let bytes = n.into() * (mem::size_of::<T>()) as u64;
        return self.buffer.slice(..bytes);
    }

    pub fn queue_write(&mut self, data: &[T], queue: &Queue) {
        let data = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buffer, 0, data);
    }
}