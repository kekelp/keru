use glam::dvec2;
use keru_draw::*;
use winit::event::*;
use winit::window::Window;

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

        let _event_consumed_by_text = self.text_window_event(ROOT_I, event, window);
        // todo keyboard events should be consumed actually.
        // but mouse events shouldn't. if a text edit gets focused, the node it's on should get focused as well.

        let event_consumed = self.ui_input(&event, window);
        if event_consumed {
            return true;
        }

        return false;
    }

    fn text_window_event(&mut self, _i: NodeI, event: &WindowEvent, window: &Window) -> bool {
        let event_consumed = self.sys.renderer.text.handle_event(event, window);

        if self.sys.renderer.text.any_text_changed() {
            if let Some(node_id) = self.sys.focused {
                self.sys.text_edit_changed_this_frame = Some(node_id);
            }
            self.sys.changes.text_changed = true;
            self.sys.new_external_events = true;
        }
        if self.sys.renderer.text.decorations_changed() {
            self.sys.new_external_events = true;
        }

        return event_consumed;
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


    /// Render a node's shape using keru_draw.
    pub(crate) fn render_node_shape_to_scene(&mut self, i: NodeI, clip_rect: Xy<[f32; 2]>) {
        let node = &self.nodes[i];

        // Get rect in normalized space (0-1)
        let animated_rect = node.get_animated_rect();

        // Convert to pixel coordinates
        let screen_size = self.sys.unifs.size;
        let x0 = (animated_rect.x[0] * screen_size.x).round();
        let y0 = (animated_rect.y[0] * screen_size.y).round();
        let x1 = (animated_rect.x[1] * screen_size.x).round();
        let y1 = (animated_rect.y[1] * screen_size.y).round();

        // Calculate hover and click darkening effects
        let clickable = if node.params.interact.senses != Sense::NONE { 1.0 } else { 0.0 };

        let t = ui_time_f32();
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

        // Get vertex colors and apply darkening
        let colors = node.params.rect.vertex_colors;
        let tl = colors.top_left_color();
        let tr = colors.top_right_color();
        let bl = colors.bottom_left_color();
        let br = colors.bottom_right_color();

        // Apply darkening to colors and convert to f32 [0-1]
        let apply_dark = |c: Color| -> [f32; 4] {
            [
                (c.r as f32 * dark) / 255.0,
                (c.g as f32 * dark) / 255.0,
                (c.b as f32 * dark) / 255.0,
                c.a as f32 / 255.0,
            ]
        };

        let tl_f = apply_dark(tl);
        let tr_f = apply_dark(tr);
        let bl_f = apply_dark(bl);
        let _br_f = apply_dark(br);

        let is_solid = tl == tr && tl == bl && tl == br;

        // Determine gradient direction by checking which corners differ
        // Old vello code used gradient from (x0, y1) bottom-left to (x1, y0) top-right
        let (gradient_start_color, gradient_end_color, gradient_angle) = if !is_solid {
            // Check if it's a horizontal, vertical, or diagonal gradient
            let is_horizontal = tl == bl && tr == br;
            let is_vertical = tl == tr && bl == br;

            if is_horizontal {
                // Horizontal gradient: left to right
                (tl_f, tr_f, 0.0) // 0 degrees = left to right
            } else if is_vertical {
                // Vertical gradient: top to bottom
                (tl_f, bl_f, std::f32::consts::FRAC_PI_2) // 90 degrees = top to bottom
            } else {
                // Diagonal gradient: use bottom-left to top-right (matching old vello behavior)
                // vello used (x0, y1) to (x1, y0) which is bottom-left to top-right
                (bl_f, tr_f, -std::f32::consts::FRAC_PI_4) // -45 degrees = bottom-left to top-right
            }
        } else {
            (tl_f, tl_f, 0.0)
        };

        // Convert clip rect to pixel coordinates
        let x_clip = [
            clip_rect.x[0] * screen_size.x,
            clip_rect.x[1] * screen_size.x,
        ];
        let y_clip = [
            clip_rect.y[0] * screen_size.y,
            clip_rect.y[1] * screen_size.y,
        ];

        let (border_thickness, border_color) = if let Some(stroke) = node.params.rect.stroke {
            let c = stroke.color;
            let border_color = [
                c.r as f32 / 255.0,
                c.g as f32 / 255.0,
                c.b as f32 / 255.0,
                c.a as f32 / 255.0,
            ];
            (stroke.width, Some(border_color))
        } else {
            (0.0, None)
        };

        // Render based on shape type
        match &node.params.rect.shape {
            Shape::Rectangle { corner_radius } => {
                let corner_radius = *corner_radius;

                // Check if one dimension is zero (for line rendering)
                let width = x1 - x0;
                let height = y1 - y0;
                let is_horizontal_line = height == 0.0 && width > 0.0;
                let is_vertical_line = width == 0.0 && height > 0.0;

                if is_horizontal_line || is_vertical_line {
                    // Draw as a segment
                    let thickness = if border_thickness > 0.0 {
                        border_thickness
                    } else {
                        1.0
                    };

                    if is_solid {
                        self.sys.renderer.draw_segment(
                            [x0, y0],
                            [x1, y1],
                            thickness,
                            tl_f,
                            x_clip,
                            y_clip,
                            None,
                        );
                    } else {
                        // For gradients on lines, use calculated gradient colors
                        self.sys.renderer.draw_segment_gradient(
                            [x0, y0],
                            [x1, y1],
                            thickness,
                            gradient_start_color,
                            gradient_end_color,
                            x_clip,
                            y_clip,
                            None,
                        );
                    }
                } else {
                    // Normal rectangle rendering
                    let top_left = [x0, y0];
                    let size = [x1 - x0, y1 - y0];

                    // If there's a border with non-zero thickness, use border color; otherwise use fill color
                    let use_border_color = border_thickness > 0.0 && border_color.is_some();

                    if is_solid {
                        let color = if use_border_color { border_color.unwrap() } else { tl_f };
                        self.sys.renderer.draw_box(
                            top_left,
                            size,
                            corner_radius,
                            border_thickness,
                            color,
                            x_clip,
                            y_clip,
                        );
                    } else {
                        // Use calculated gradient direction
                        let (start_color, end_color) = if use_border_color {
                            let bc = border_color.unwrap();
                            (bc, bc)
                        } else {
                            (gradient_start_color, gradient_end_color)
                        };
                        self.sys.renderer.draw_box_gradient(
                            top_left,
                            size,
                            corner_radius,
                            border_thickness,
                            start_color,
                            end_color,
                            gradient_angle,
                            x_clip,
                            y_clip,
                        );
                    }
                }
            }
            Shape::Circle => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);

                if is_solid {
                    self.sys.renderer.draw_circle(
                        [cx, cy],
                        radius,
                        tl_f,
                        x_clip,
                        y_clip,
                    );
                } else {
                    // For circles, determine if we should use linear or radial gradient
                    // Radial gradient for actual color difference, linear for directional
                    let is_horizontal = tl == bl && tr == br;
                    let is_vertical = tl == tr && bl == br;

                    if is_horizontal || is_vertical {
                        // Use linear gradient
                        self.sys.renderer.draw_circle_gradient(
                            [cx, cy],
                            radius,
                            gradient_start_color,
                            gradient_end_color,
                            1, // linear gradient
                            gradient_angle,
                            x_clip,
                            y_clip,
                        );
                    } else {
                        // Use radial gradient for diagonal
                        self.sys.renderer.draw_circle_gradient(
                            [cx, cy],
                            radius,
                            gradient_start_color,
                            gradient_end_color,
                            2, // radial gradient
                            0.0,
                            x_clip,
                            y_clip,
                        );
                    }
                }
            }
            Shape::Ring { width } => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let outer_radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                let inner_radius = (outer_radius - *width).max(0.0);

                if is_solid {
                    self.sys.renderer.draw_ring(
                        [cx, cy],
                        inner_radius,
                        outer_radius,
                        tl_f,
                        x_clip,
                        y_clip,
                    );
                } else {
                    // For rings, determine if we should use linear or radial gradient
                    let is_horizontal = tl == bl && tr == br;
                    let is_vertical = tl == tr && bl == br;

                    if is_horizontal || is_vertical {
                        // Use linear gradient
                        self.sys.renderer.draw_ring_gradient(
                            [cx, cy],
                            inner_radius,
                            outer_radius,
                            gradient_start_color,
                            gradient_end_color,
                            1, // linear gradient
                            gradient_angle,
                            x_clip,
                            y_clip,
                        );
                    } else {
                        // Use radial gradient for diagonal
                        self.sys.renderer.draw_ring_gradient(
                            [cx, cy],
                            inner_radius,
                            outer_radius,
                            gradient_start_color,
                            gradient_end_color,
                            2, // radial gradient
                            0.0,
                            x_clip,
                            y_clip,
                        );
                    }
                }
            }
        }
    }

    /// Renders the UI into a render pass.
    pub fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
    ) {
        // todo think harder
        if self.sys.changes.should_rebuild_render_data || self.sys.anim_render_timer.is_live() {
            self.rebuild_render_data();
        }

        self.sys.renderer.render(render_pass);

        self.sys.renderer.text.clear_changes();
        self.sys.changes.need_rerender = false;
    }

    /// Convenience function that creates a render pass, renders into it, and presents to the screen.
    ///
    /// Panics if the current surface texture can't be obtained from `surface`.
    pub fn autorender(
        &mut self,
        surface: &wgpu::Surface,
        background_color: wgpu::Color,
    ) {
        let output = surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.sys.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("keru_draw autorender render encoder"),
        });

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

        self.sys.queue.submit(std::iter::once(encoder.finish()));

        output.present();
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
}

#[derive(Clone, Debug)]
pub enum ImageRef {
    Raster(LoadedImage),
    Svg {
        loaded: LoadedImage,
        data: &'static [u8],
        rasterized_width: u32,
        rasterized_height: u32,
    },
}
