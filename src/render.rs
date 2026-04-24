use glam::vec2;
use keru_draw::*;
use winit::event::*;

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
    pub fn window_event(&mut self, event: &winit::event::WindowEvent, window: &winit::window::Window) -> bool {
        self.sys.mouse_input.window_event(event);
        self.sys.key_input.window_event(event);

        let _event_consumed_by_text = self.text_window_event(event, window);
        // todo keyboard events should be consumed actually.
        // but mouse events shouldn't. if a text edit gets focused, the node it's on should get focused as well.

        let event_consumed = self.ui_input(&event, window);
        if event_consumed {
            return true;
        }

        return false;
    }

    fn text_window_event(
        &mut self,
        event: &winit::event::WindowEvent,
        window: &winit::window::Window
    ) -> bool {
        // Pass events to keru_text so it can register windows and track dimensions.
        // This is essential for keru_text to set the screen resolution correctly.
        self.sys.renderer.text.handle_event(event, window);
        // todo: see if keyboard events are consumed by a text box?

        if self.sys.renderer.text.needs_rerender() {
            self.sys.changes.should_rebuild_render_data = true;
        }
        return false;
    }

    pub fn ui_input(&mut self, event: &winit::event::WindowEvent, window: &winit::window::Window) -> bool {
        match event {
            WindowEvent::RedrawRequested => {
                self.new_redraw_requested_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.resolve_hover();

                let last_cursor_pos = self.sys.mouse_input.prev_cursor_position;
                if vec2(position.x as f32, position.y as f32) != last_cursor_pos {
                    let mut anything_was_hovered = false;
                    for id in &self.sys.hovered {
                        if let Some(i) = self.sys.nodes.get_by_id(*id) {
                            if self.sys.nodes[i].params.interact.senses.contains(Sense::HOVER) {
                                anything_was_hovered = true;
                                break;
                            }
                        }
                    }

                    let mut anything_was_dragged = false;
                    for (id, _) in self.sys.mouse_input.currently_dragging() {
                        if let Some(i) = self.sys.nodes.get_by_id(*id) {
                            if self.sys.nodes[i].params.interact.senses.contains(Sense::DRAG) {
                                anything_was_dragged = true;
                                break;
                            }
                        }
                    }

                    if anything_was_hovered || anything_was_dragged {
                        self.set_new_ui_input();
                    }
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                match state {
                    ElementState::Pressed => {
                        return self.handle_mouse_press(*button, window);
                    }
                    ElementState::Released => {
                        self.handle_mouse_release(*button);
                        return false;
                    }
                }
            }
            WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                // In release mode, only trigger UI update for listened keys
                #[cfg(not(debug_assertions))]
                {
                    let should_update = !self.sys.filter_listened_keys
                        || self.sys.listened_keys.contains(&event.logical_key);
                    if should_update {
                        self.set_new_ui_input();
                    }
                }
                // In debug mode, always trigger UI update (filter is only used for warnings)
                #[cfg(debug_assertions)]
                self.set_new_ui_input();

                if !is_synthetic {
                    return self.handle_keyboard_event(event);
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
        false
    }

    /// Render a node's shape using keru_draw.
    pub(crate) fn render_node_shape_to_scene(&mut self, i: NodeI, texture: Option<LoadedImage>, debug_box: bool) {
        let node = &self.sys.nodes[i];

        let blur = node.params.blur.unwrap_or(0.0);
        // Get rect in normalized space (0-1)
        let animated_rect = node.get_animated_rect();

        // Convert to pixel coordinates
        // Round to screen pixels using transform scale
        let screen_size = self.sys.size;
        let scale = node.accumulated_transform.scale;
        let x0 = (animated_rect.x[0] * screen_size.x * scale).round() / scale;
        let y0 = (animated_rect.y[0] * screen_size.y * scale).round() / scale;
        let x1 = (animated_rect.x[1] * screen_size.x * scale).round() / scale;
        let y1 = (animated_rect.y[1] * screen_size.y * scale).round() / scale;

        // Calculate hover and click darkening effects
        let clickable = if node.params.interact.senses != Sense::NONE { 1.0 } else { 0.0 };

        let t = self.sys.t;
        let t_since_hover = (t - node.hover_timestamp) * 10.0;
        let hover = if node.hovered {
            t_since_hover.clamp(0.0, 1.0) * clickable
        } else {
            (1.0 - t_since_hover.clamp(0.0, 1.0)) * if t_since_hover < 1.0 { 1.0 } else { 0.0 } * clickable
        };

        let t_since_click = (t - node.last_click) * 4.1;
        let click = (1.0 - t_since_click.clamp(0.0, 1.0)) * if t_since_click < 1.0 { 1.0 } else { 0.0 } * clickable;

        let dark_hover = 1.0 - hover * 0.32;
        let dark_click = 1.0 - click * 0.78;
        let dark = dark_click.min(dark_hover);

        // Apply darkening to fill
        let apply_dark = |c: Color| -> Color {
            Color::new(c.r * dark, c.g * dark, c.b * dark, c.a)
        };
        let apply_dark_fill = |f: ColorFill| -> ColorFill {
            match f {
                ColorFill::Color(color) => ColorFill::Color(apply_dark(color)),
                ColorFill::Gradient(g) => ColorFill::Gradient(keru_draw::Gradient {
                    color_start: apply_dark(g.color_start),
                    color_end: apply_dark(g.color_end),
                    gradient_type: g.gradient_type,
                    angle: g.angle,
                }),
            }
        };

        let fill = apply_dark_fill(node.params.color);

        // Get stroke info
        let stroke = if debug_box {
            Some(Stroke::new(3.0).with_color(Color::DEBUG_RED))
        } else {
            node.params.stroke
        };

        let shape = if debug_box {
            &Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 5.0 }
        } else {
            &node.params.shape
        };

        // Check if fill is visible (alpha > 0)
        let fill_visible = !debug_box && match node.params.color {
            ColorFill::Color(c) => c.a > 0.0,
            ColorFill::Gradient(g) => g.color_start.a > 0.0 || g.color_end.a > 0.0,
        };

        let shadow_color = |s: Shadow| -> Color {
            s.color.unwrap_or_else(|| {
                let base = match node.params.color {
                    ColorFill::Color(c) => c,
                    ColorFill::Gradient(g) => g.color_start,
                };
                Color::new(base.r * 0.3, base.g * 0.3, base.b * 0.3, base.a * 0.7)
            })
        };

        struct ShapePass {
            offset: Xy<f32>,
            blur: f32,
            fill: ColorFill,
            texture: Option<LoadedImage>,
        }

        let shadow_pass = |s: Shadow| ShapePass {
            offset: s.offset,
            blur: blur + s.blur,
            fill: ColorFill::Color(shadow_color(s)),
            texture,
        };

        let real_pass = ShapePass {
            offset: Xy::new(0.0, 0.0),
            blur,
            fill,
            texture,
        };

        let passes: [Option<ShapePass>; 3] = if debug_box {
            [None, None, Some(real_pass)]
        } else {
            [
                node.params.shadow.map(shadow_pass),
                node.params.second_shadow.map(shadow_pass),
                Some(real_pass),
            ]
        };

        // Draw all shapes, first the shadows then the real shape
        for pass in passes.into_iter().flatten() {
            let px0 = x0 + pass.offset.x;
            let py0 = y0 + pass.offset.y;
            let px1 = x1 + pass.offset.x;
            let py1 = y1 + pass.offset.y;

            match shape {
                Shape::NoShape => {}
                Shape::Rectangle { rounded_corners, corner_radius } => {
                    self.sys.renderer.draw_box(keru_draw::Box {
                        top_left: [px0, py0],
                        size: [px1 - px0, py1 - py0],
                        corner_radius: *corner_radius,
                        rounded_corners: *rounded_corners,
                        border_thickness: 0.0,
                        fill: pass.fill,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
                Shape::Circle => {
                    let cx = (px0 + px1) / 2.0;
                    let cy = (py0 + py1) / 2.0;
                    let radius = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                    self.sys.renderer.draw_circle(keru_draw::Circle {
                        center: [cx, cy],
                        radius,
                        fill: pass.fill,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
                Shape::Ring { width } => {
                    let cx = (px0 + px1) / 2.0;
                    let cy = (py0 + py1) / 2.0;
                    let outer_radius = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                    let inner_radius = (outer_radius - *width).max(0.0);
                    let dash_length = stroke.and_then(|s| if s.dash_length > 0.0 { Some(s.dash_length) } else { None });
                    self.sys.renderer.draw_ring(keru_draw::CircleRing {
                        center: [cx, cy],
                        inner_radius,
                        outer_radius,
                        fill: pass.fill,
                        texture: pass.texture,
                        dash_length,
                        dash_offset: 0.0,
                        blur: pass.blur,
                    });
                }
                Shape::Arc { start_angle, end_angle, width } => {
                    let cx = (px0 + px1) / 2.0;
                    let cy = (py0 + py1) / 2.0;
                    let radius = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                    let dash_length = stroke.and_then(|s| if s.dash_length > 0.0 { Some(s.dash_length) } else { None });
                    self.sys.renderer.draw_arc(keru_draw::CircleArc {
                        center: [cx, cy],
                        radius,
                        start_angle: *start_angle,
                        end_angle: *end_angle,
                        thickness: *width,
                        fill: pass.fill,
                        texture: pass.texture,
                        dash_length,
                        dash_offset: 0.0,
                        blur: pass.blur,
                    });
                }
                Shape::Pie { start_angle, end_angle } => {
                    let cx = (px0 + px1) / 2.0;
                    let cy = (py0 + py1) / 2.0;
                    let radius = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                    self.sys.renderer.draw_pie(keru_draw::CirclePie {
                        center: [cx, cy],
                        radius,
                        start_angle: *start_angle,
                        end_angle: *end_angle,
                        fill: pass.fill,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
                Shape::Segment { start, end, dash_length } => {
                    let start_x = px0 + start.0 * (px1 - px0);
                    let start_y = py0 + start.1 * (py1 - py0);
                    let end_x = px0 + end.0 * (px1 - px0);
                    let end_y = py0 + end.1 * (py1 - py0);
                    let thickness = stroke.map(|s| s.width).unwrap_or(1.0);
                    self.sys.renderer.draw_segment(keru_draw::Segment {
                        start: [start_x, start_y],
                        end: [end_x, end_y],
                        thickness,
                        fill: pass.fill,
                        dash_length: *dash_length,
                        dash_offset: 0.0,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
                Shape::HorizontalLine => {
                    let thickness = stroke.map(|s| s.width).unwrap_or(1.0);
                    let cy = (py0 + py1) / 2.0;
                    let dash_length = stroke.and_then(|s| if s.dash_length > 0.0 { Some(s.dash_length) } else { None });
                    self.sys.renderer.draw_segment(keru_draw::Segment {
                        start: [px0, cy],
                        end: [px1, cy],
                        thickness,
                        fill: pass.fill,
                        dash_length,
                        dash_offset: 0.0,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
                Shape::Hexagon { size, rotation } => {
                    let cx = (px0 + px1) / 2.0;
                    let cy = (py0 + py1) / 2.0;
                    let max_radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                    let actual_size = max_radius * size;

                    if fill_visible {
                        self.sys.renderer.draw_hexagon(keru_draw::Hexagon {
                            center: [cx, cy],
                            size: actual_size,
                            rotation: *rotation,
                            fill: pass.fill,
                            stroke_thickness: 0.0,
                            texture: pass.texture,
                            blur: pass.blur,
                        });
                    }
                }
                Shape::VerticalLine => {
                    let thickness = stroke.map(|s| s.width).unwrap_or(1.0);
                    let cx = (px0 + px1) / 2.0;
                    let dash_length = stroke.and_then(|s| if s.dash_length > 0.0 { Some(s.dash_length) } else { None });
                    self.sys.renderer.draw_segment(keru_draw::Segment {
                        start: [cx, py0],
                        end: [cx, py1],
                        thickness,
                        fill: pass.fill,
                        dash_length,
                        dash_offset: 0.0,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
                Shape::Triangle { rotation, width } => {
                    let cx = (px0 + px1) / 2.0;
                    let cy = (py0 + py1) / 2.0;
                    let rect_width = px1 - px0;
                    let rect_height = py1 - py0;
                    let radius = rect_width.min(rect_height) / 2.0;

                    let cos_r = rotation.cos();
                    let sin_r = rotation.sin();
                    let tip_dist = radius;
                    let base_back = radius * 0.5;
                    let base_half_width = radius * 0.866 * width;

                    let p0_x = cx + tip_dist * cos_r;
                    let p0_y = cy + tip_dist * sin_r;
                    let perp_x = -sin_r;
                    let perp_y = cos_r;
                    let p1_x = cx - base_back * cos_r + base_half_width * perp_x;
                    let p1_y = cy - base_back * sin_r + base_half_width * perp_y;
                    let p2_x = cx - base_back * cos_r - base_half_width * perp_x;
                    let p2_y = cy - base_back * sin_r - base_half_width * perp_y;

                    self.sys.renderer.draw_triangle(keru_draw::Triangle {
                        p0: [p0_x, p0_y],
                        p1: [p1_x, p1_y],
                        p2: [p2_x, p2_y],
                        fill: pass.fill,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
                Shape::SquareGrid { lattice_size, offset, line_thickness } => {
                    let grid_color = match pass.fill {
                        ColorFill::Color(c) => c,
                        ColorFill::Gradient(g) => g.color_start,
                    };
                    self.sys.renderer.draw_grid(keru_draw::Grid {
                        top_left: [px0, py0],
                        size: [px1 - px0, py1 - py0],
                        lattice_size: *lattice_size,
                        offset: [offset.0, offset.1],
                        line_thickness: *line_thickness,
                        color: grid_color,
                        grid_type: keru_draw::GridType::Square,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
                Shape::HexGrid { lattice_size, offset, line_thickness } => {
                    let grid_color = match pass.fill {
                        ColorFill::Color(c) => c,
                        ColorFill::Gradient(g) => g.color_start,
                    };
                    self.sys.renderer.draw_grid(keru_draw::Grid {
                        top_left: [px0, py0],
                        size: [px1 - px0, py1 - py0],
                        lattice_size: *lattice_size,
                        offset: [offset.0, offset.1],
                        line_thickness: *line_thickness,
                        color: grid_color,
                        grid_type: keru_draw::GridType::Hexagonal,
                        texture: pass.texture,
                        blur: pass.blur,
                    });
                }
            }
        }

        // Draw strokes
        match shape {
            Shape::NoShape => {}
            Shape::Rectangle { rounded_corners, corner_radius } => {
                let corner_radius = *corner_radius;
                let width = x1 - x0;
                let height = y1 - y0;
                if let Some(stroke) = stroke {
                    let stroke_fill = apply_dark_fill(stroke.color);
                    if stroke.dash_length > 0.0 {
                        // Dashed stroke
                        let stroke_color = match stroke_fill {
                            ColorFill::Color(c) => c,
                            ColorFill::Gradient(g) => g.color_start,
                        };
                        self.sys.renderer.draw_dashed_box_outline(keru_draw::DashedBoxOutline {
                            top_left: [x0, y0],
                            size: [width, height],
                            corner_radius,
                            thickness: stroke.width,
                            color: stroke_color,
                            dash_length: stroke.dash_length,
                            blur,
                        });
                    } else {
                        // Solid stroke
                        self.sys.renderer.draw_box(keru_draw::Box {
                            top_left: [x0, y0],
                            size: [width, height],
                            corner_radius,
                            rounded_corners: *rounded_corners,
                            border_thickness: stroke.width,
                            fill: stroke_fill,
                            texture: None,
                            blur,
                        });
                    }
                }
            }
            Shape::Circle => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                if let Some(stroke) = stroke {
                    let stroke_fill = apply_dark_fill(stroke.color);
                    let dash_length = if stroke.dash_length > 0.0 { Some(stroke.dash_length) } else { None };
                    self.sys.renderer.draw_ring(keru_draw::CircleRing {
                        center: [cx, cy],
                        inner_radius: radius - stroke.width * 0.5,
                        outer_radius: radius + stroke.width * 0.5,
                        fill: stroke_fill,
                        texture: None,
                        dash_length,
                        dash_offset: 0.0,
                        blur,
                    });
                }
            }
            Shape::Ring { .. } | Shape::Arc { .. } | Shape::Pie { .. } | Shape::Segment { .. } | Shape::HorizontalLine => {}
            Shape::VerticalLine | Shape::Triangle { .. } | Shape::SquareGrid { .. } | Shape::HexGrid { .. } => {}
            Shape::Hexagon { size, rotation } => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let max_radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                let actual_size = max_radius * size;

                if let Some(stroke) = stroke {
                    let stroke_fill = apply_dark_fill(stroke.color);
                    if stroke.dash_length > 0.0 {
                        // Dashed stroke
                        let stroke_color = match stroke_fill {
                            ColorFill::Color(c) => c,
                            ColorFill::Gradient(g) => g.color_start,
                        };
                        self.sys.renderer.draw_dashed_hexagon_outline(keru_draw::DashedHexagonOutline {
                            center: [cx, cy],
                            size: actual_size,
                            rotation: *rotation,
                            thickness: stroke.width,
                            color: stroke_color,
                            dash_length: stroke.dash_length,
                            blur,
                        });
                    } else {
                        // Solid stroke
                        self.sys.renderer.draw_hexagon(keru_draw::Hexagon {
                            center: [cx, cy],
                            size: actual_size,
                            rotation: *rotation,
                            fill: stroke_fill,
                            stroke_thickness: stroke.width,
                            texture: None,
                            blur,
                        });
                    }
                }
            }
        }
    }

    /// Renders the UI into a render pass.
    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass) {
        // It makes sense to update it here because it's only used for render effects.
        // If it was used for other things, it would be better to update it in something like begin_frame,
        // but begin_frame doesn't work because it's normal to do rerenders without rerunning begin_frame and the update. 
        self.sys.t = T0.elapsed().as_secs_f32();

        // todo think harder
        if self.sys.changes.should_rebuild_render_data || self.sys.anim_render_timer.is_live() {
            self.rebuild_render_data();
        }

        self.sys.renderer.render(render_pass);

        self.sys.changes.need_rerender = false;
    }

    /// Convenience function that creates a render pass, renders into it, and presents to the screen.
    ///
    /// Panics if the current surface texture can't be obtained from `surface`.
    /// 
    /// Use [`Ui::render()`] to render the Ui into an existing render pass.
    pub fn autorender(&mut self, surface: &wgpu::Surface, background_color: wgpu::Color) {
        let output = surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.sys.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("keru_draw autorender render encoder"),
        });

        let query = self.sys.renderer.gpu_profiler.begin_query("Render", &mut encoder);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("keru_draw autorender render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(background_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.render(&mut render_pass);
        }

        self.sys.renderer.gpu_profiler.end_query(&mut encoder, query);
        self.sys.renderer.gpu_profiler.resolve_queries(&mut encoder);

        self.sys.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        self.sys.renderer.gpu_profiler.end_frame().unwrap();

        #[cfg(debug_assertions)]
        {
            let profiling_data = self.sys.renderer.gpu_profiler.process_finished_frame(self.sys.queue.get_timestamp_period());
            if let Some(profiling_data) = profiling_data {
                for p in profiling_data {
                    if let Some(time) = p.time {
                        let dur = std::time::Duration::from_secs_f64(time.end - time.start);
                        log::info!("Gpu time ({}): {:?}", p.label, dur);
                    }
                }
            }
        }
    }

    /// Returns `true` if the `Ui` needs to be rerendered.
    ///
    /// If this is true, you should call [`Ui::render`] as soon as possible to display the updated GUI state on the screen.
    pub fn should_rerender(&mut self) -> bool {
        return self.sys.changes.need_rerender
            || self.sys.anim_render_timer.is_live()
            || self.sys.renderer.text.needs_rerender()
            || self.sys.changes.should_rebuild_render_data;
    }

    /// Set up the render pass for custom rendering using the render plan.
    ///
    /// This must be called before using `render_range()` to draw individual ranges.
    /// It uploads all GPU data and sets up the render pipeline and bind groups.
    ///
    /// After calling this, you can call `render_range()` multiple times to draw
    /// specific ranges of instances, interleaving with your own custom rendering.
    // todo: deduplicate and simplify this stuff
    pub fn begin_custom_render(&mut self) {
        self.sys.t = T0.elapsed().as_secs_f32(); // todo: maybe deduplicate better

        // Rebuild render data if needed
        if self.sys.changes.should_rebuild_render_data || self.sys.anim_render_timer.is_live() {
            self.rebuild_render_data();
        }

        self.sys.renderer.load_to_gpu();
    }

    /// Render a specific range of instances into a render pass.
    ///
    /// This is useful for custom rendering where you want to interleave
    /// Keru's rendering with your own custom drawing code using the render plan.
    ///
    /// You must call `setup_render_pass()` before calling this method.
    pub fn render_range(&mut self, render_pass: &mut wgpu::RenderPass, range: KeruElementRange) {
        self.sys.renderer.render_range(render_pass, range.0);
    }

    /// Finish rendering after using custom render plan.
    ///
    /// Call this after you're done with all render_range() calls to clean up state.
    pub fn finish_custom_render(&mut self) {
        self.sys.changes.need_rerender = false;
    }

    /// Submit command buffer to the GPU queue.
    ///
    /// This is a convenience method for custom rendering loops.
    pub fn submit_commands(&mut self, command_buffer: wgpu::CommandBuffer) {
        self.sys.queue.submit(std::iter::once(command_buffer));
    }
}

#[derive(Clone, Debug)]
pub enum ImageRef {
    Raster(LoadedImage),
    Svg(LoadedImage),
}
