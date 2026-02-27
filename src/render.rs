use glam::vec2;
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
                self.resolve_hover();

                let last_cursor_pos = self.sys.mouse_input.prev_cursor_position;
                if vec2(position.x as f32, position.y as f32) != last_cursor_pos {
                    let has_hover_sense = self.sys.hovered.iter()
                        .filter_map(|id| self.nodes.get_by_id_ick(id).map(|(node, _)| node))
                        .any(|node| node.params.interact.senses.contains(Sense::HOVER));

                    let has_drag = self.sys.mouse_input.currently_dragging()
                        .filter_map(|(id, _)| self.nodes.get_by_id_ick(id).map(|(node, _)| node))
                        .any(|node| node.params.interact.senses.contains(Sense::DRAG));

                    if has_hover_sense || has_drag {
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
    pub(crate) fn render_node_shape_to_scene(&mut self, i: NodeI, clip_rect: Xy<[f32; 2]>, texture: Option<LoadedImage>, debug_box: bool) {
        let node = &self.nodes[i];

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

        // Get vertex colors and apply darkening
        let colors = node.params.color;
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

        let (border_thickness, border_color) = if let Some(stroke) = node.params.stroke {
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

        // Override colors for debug box
        let (tl_f, is_solid, border_thickness, border_color) = if debug_box {
            let c = Color::KERU_DEBUG_RED;
            let border_color_f32 = [
                c.r as f32 / 255.0,
                c.g as f32 / 255.0,
                c.b as f32 / 255.0,
                c.a as f32 / 255.0,
            ];
            ([0.0, 0.0, 0.0, 0.0], true, 3.0, Some(border_color_f32))
        } else {
            (tl_f, is_solid, border_thickness, border_color)
        };

        let shape = if debug_box {
            &Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 5.0 }
        } else {
            &node.params.shape
        };

        // Render based on shape type
        match shape {
            Shape::NoShape => {}
            Shape::Rectangle { rounded_corners, corner_radius } => {
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

                    let fill = if is_solid {
                        keru_draw::Fill::Solid(tl_f)
                    } else {
                        keru_draw::Fill::Gradient {
                            color_start: gradient_start_color,
                            color_end: gradient_end_color,
                            gradient_type: keru_draw::GradientType::Linear,
                            angle: gradient_angle,
                        }
                    };

                    self.sys.renderer.draw_segment(keru_draw::Segment {
                        start: [x0, y0],
                        end: [x1, y1],
                        thickness,
                        fill,
                        x_clip,
                        y_clip: y_clip,
                        dash_length: None,
                        texture,
                    });
                } else {
                    // Normal rectangle rendering
                    let top_left = [x0, y0];
                    let size = [x1 - x0, y1 - y0];

                    // If there's a border with non-zero thickness, use border color; otherwise use fill color
                    let use_border_color = border_thickness > 0.0 && border_color.is_some();

                    let fill = if is_solid {
                        let color = if use_border_color { border_color.unwrap() } else { tl_f };
                        keru_draw::Fill::Solid(color)
                    } else {
                        let (start_color, end_color) = if use_border_color {
                            let bc = border_color.unwrap();
                            (bc, bc)
                        } else {
                            (gradient_start_color, gradient_end_color)
                        };
                        keru_draw::Fill::Gradient {
                            color_start: start_color,
                            color_end: end_color,
                            gradient_type: keru_draw::GradientType::Linear,
                            angle: gradient_angle,
                        }
                    };

                    self.sys.renderer.draw_box(keru_draw::Box {
                        top_left,
                        size,
                        corner_radius,
                        rounded_corners: *rounded_corners,
                        border_thickness,
                        fill,
                        x_clip,
                        y_clip,
                        texture,
                    });
                }
            }
            Shape::Circle => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);

                let fill = if is_solid {
                    keru_draw::Fill::Solid(tl_f)
                } else {
                    // For circles, determine if we should use linear or radial gradient
                    let is_horizontal = tl == bl && tr == br;
                    let is_vertical = tl == tr && bl == br;

                    if is_horizontal || is_vertical {
                        // Use linear gradient
                        keru_draw::Fill::Gradient {
                            color_start: gradient_start_color,
                            color_end: gradient_end_color,
                            gradient_type: keru_draw::GradientType::Linear,
                            angle: gradient_angle,
                        }
                    } else {
                        // Use radial gradient for diagonal
                        keru_draw::Fill::Gradient {
                            color_start: gradient_start_color,
                            color_end: gradient_end_color,
                            gradient_type: keru_draw::GradientType::Radial,
                            angle: 0.0,
                        }
                    }
                };

                self.sys.renderer.draw_circle(keru_draw::Circle {
                    center: [cx, cy],
                    radius,
                    fill,
                    x_clip,
                    y_clip,
                    texture,
                });
            }
            Shape::Ring { width } => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let outer_radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                let inner_radius = (outer_radius - *width).max(0.0);

                let fill = if is_solid {
                    keru_draw::Fill::Solid(tl_f)
                } else {
                    // For rings, determine if we should use linear or radial gradient
                    let is_horizontal = tl == bl && tr == br;
                    let is_vertical = tl == tr && bl == br;

                    if is_horizontal || is_vertical {
                        // Use linear gradient
                        keru_draw::Fill::Gradient {
                            color_start: gradient_start_color,
                            color_end: gradient_end_color,
                            gradient_type: keru_draw::GradientType::Linear,
                            angle: gradient_angle,
                        }
                    } else {
                        // Use radial gradient for diagonal
                        keru_draw::Fill::Gradient {
                            color_start: gradient_start_color,
                            color_end: gradient_end_color,
                            gradient_type: keru_draw::GradientType::Radial,
                            angle: 0.0,
                        }
                    }
                };

                self.sys.renderer.draw_ring(keru_draw::Ring {
                    center: [cx, cy],
                    inner_radius,
                    outer_radius,
                    fill,
                    x_clip,
                    y_clip,
                    texture,
                });
            }
            Shape::Arc { start_angle, end_angle, width } => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);

                let fill = if is_solid {
                    keru_draw::Fill::Solid(tl_f)
                } else {
                    keru_draw::Fill::Gradient {
                        color_start: gradient_start_color,
                        color_end: gradient_end_color,
                        gradient_type: keru_draw::GradientType::Linear,
                        angle: gradient_angle,
                    }
                };

                self.sys.renderer.draw_arc(keru_draw::CircleArc {
                    center: [cx, cy],
                    radius,
                    start_angle: *start_angle,
                    end_angle: *end_angle,
                    thickness: *width,
                    fill,
                    x_clip,
                    y_clip,
                    texture,
                });
            }
            Shape::Pie { start_angle, end_angle } => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);

                let fill = if is_solid {
                    keru_draw::Fill::Solid(tl_f)
                } else {
                    keru_draw::Fill::Gradient {
                        color_start: gradient_start_color,
                        color_end: gradient_end_color,
                        gradient_type: keru_draw::GradientType::Linear,
                        angle: gradient_angle,
                    }
                };

                self.sys.renderer.draw_pie(keru_draw::CirclePie {
                    center: [cx, cy],
                    radius,
                    start_angle: *start_angle,
                    end_angle: *end_angle,
                    fill,
                    x_clip,
                    y_clip,
                    texture,
                });
            }
            Shape::Segment { start, end, dash_length } => {
                // Convert normalized coordinates to pixel coordinates
                let start_x = x0 + start.0 * (x1 - x0);
                let start_y = y0 + start.1 * (y1 - y0);
                let end_x = x0 + end.0 * (x1 - x0);
                let end_y = y0 + end.1 * (y1 - y0);

                let thickness = if let Some(stroke) = node.params.stroke {
                    stroke.width
                } else {
                    1.0
                };


                let fill = if is_solid {
                    keru_draw::Fill::Solid(tl_f)
                } else {
                    keru_draw::Fill::Gradient {
                        color_start: gradient_start_color,
                        color_end: gradient_end_color,
                        gradient_type: keru_draw::GradientType::Linear,
                        angle: gradient_angle,
                    }
                };

                self.sys.renderer.draw_segment(keru_draw::Segment {
                    start: [start_x, start_y],
                    end: [end_x, end_y],
                    thickness,
                    fill,
                    x_clip,
                    y_clip,
                    dash_length: *dash_length,
                    texture,
                });
            }
            Shape::HorizontalLine => {
                let thickness = if let Some(stroke) = node.params.stroke {
                    stroke.width
                } else {
                    1.0
                };

                let cy = (y0 + y1) / 2.0;

                let dash_length = node.params.stroke.and_then(|s| {
                    if s.dash_length > 0.0 { Some(s.dash_length) } else { None }
                });

                let fill = if is_solid {
                    keru_draw::Fill::Solid(tl_f)
                } else {
                    keru_draw::Fill::Gradient {
                        color_start: gradient_start_color,
                        color_end: gradient_end_color,
                        gradient_type: keru_draw::GradientType::Linear,
                        angle: gradient_angle,
                    }
                };

                self.sys.renderer.draw_segment(keru_draw::Segment {
                    start: [x0, cy],
                    end: [x1, cy],
                    thickness,
                    fill,
                    x_clip,
                    y_clip,
                    dash_length,
                    texture,
                });
            }
            Shape::VerticalLine => {
                let thickness = if let Some(stroke) = node.params.stroke {
                    stroke.width
                } else {
                    1.0
                };

                let cx = (x0 + x1) / 2.0;

                let dash_length = node.params.stroke.and_then(|s| {
                    if s.dash_length > 0.0 { Some(s.dash_length) } else { None }
                });

                let fill = if is_solid {
                    keru_draw::Fill::Solid(tl_f)
                } else {
                    keru_draw::Fill::Gradient {
                        color_start: gradient_start_color,
                        color_end: gradient_end_color,
                        gradient_type: keru_draw::GradientType::Linear,
                        angle: gradient_angle,
                    }
                };

                self.sys.renderer.draw_segment(keru_draw::Segment {
                    start: [cx, y0],
                    end: [cx, y1],
                    thickness,
                    fill,
                    x_clip,
                    y_clip,
                    dash_length,
                    texture,
                });
            }
            Shape::Triangle { rotation, width } => {
                // Generate isosceles triangle with one vertex pointing in rotation direction
                // width = 1.0 gives equilateral, <1.0 narrower, >1.0 wider
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let rect_width = x1 - x0;
                let rect_height = y1 - y0;
                let radius = rect_width.min(rect_height) / 2.0;

                let cos_r = rotation.cos();
                let sin_r = rotation.sin();

                // For equilateral triangle inscribed in circle:
                // - Tip is at distance r from center
                // - Base vertices are at (-0.5r, ±0.866r) in local coords
                let tip_dist = radius;
                let base_back = radius * 0.5;
                let base_half_width = radius * 0.866 * width; // sqrt(3)/2 ≈ 0.866

                // Tip vertex (pointing in rotation direction)
                let p0_x = cx + tip_dist * cos_r;
                let p0_y = cy + tip_dist * sin_r;

                // Perpendicular direction (rotate by 90°)
                let perp_x = -sin_r;
                let perp_y = cos_r;

                // Base vertices (behind and to the sides)
                let p1_x = cx - base_back * cos_r + base_half_width * perp_x;
                let p1_y = cy - base_back * sin_r + base_half_width * perp_y;
                let p2_x = cx - base_back * cos_r - base_half_width * perp_x;
                let p2_y = cy - base_back * sin_r - base_half_width * perp_y;

                let fill = if is_solid {
                    keru_draw::Fill::Solid(tl_f)
                } else {
                    keru_draw::Fill::Gradient {
                        color_start: gradient_start_color,
                        color_end: gradient_end_color,
                        gradient_type: keru_draw::GradientType::Linear,
                        angle: gradient_angle,
                    }
                };

                self.sys.renderer.draw_triangle(keru_draw::Triangle {
                    p0: [p0_x, p0_y],
                    p1: [p1_x, p1_y],
                    p2: [p2_x, p2_y],
                    fill,
                    x_clip,
                    y_clip,
                    texture,
                });
            }
            Shape::SquareGrid { lattice_size, offset, line_thickness } => {
                let top_left = [x0, y0];
                let size = [x1 - x0, y1 - y0];

                // For grid, we use the fill color (not stroke)
                self.sys.renderer.draw_grid(keru_draw::Grid {
                    top_left,
                    size,
                    lattice_size: *lattice_size,
                    offset: [offset.0, offset.1],
                    line_thickness: *line_thickness,
                    color: tl_f,
                    grid_type: keru_draw::GridType::Square,
                    x_clip,
                    y_clip,
                    texture,
                });
            }
            Shape::HexGrid { lattice_size, offset, line_thickness } => {
                let top_left = [x0, y0];
                let size = [x1 - x0, y1 - y0];

                // For hex grid, we use the fill color (not stroke)
                self.sys.renderer.draw_grid(keru_draw::Grid {
                    top_left,
                    size,
                    lattice_size: *lattice_size,
                    offset: [offset.0, offset.1],
                    line_thickness: *line_thickness,
                    color: tl_f,
                    grid_type: keru_draw::GridType::Hexagonal,
                    x_clip,
                    y_clip,
                    texture,
                });
            }
            Shape::Hexagon { size, rotation } => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let max_radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                let actual_size = max_radius * size;

                // If there's a border with non-zero thickness, use border color; otherwise use fill color
                let use_border_color = border_thickness > 0.0 && border_color.is_some();

                let fill = if is_solid {
                    let color = if use_border_color { border_color.unwrap() } else { tl_f };
                    keru_draw::Fill::Solid(color)
                } else {
                    let (start_color, end_color) = if use_border_color {
                        let bc = border_color.unwrap();
                        (bc, bc)
                    } else {
                        (gradient_start_color, gradient_end_color)
                    };
                    keru_draw::Fill::Gradient {
                        color_start: start_color,
                        color_end: end_color,
                        gradient_type: keru_draw::GradientType::Linear,
                        angle: gradient_angle,
                    }
                };

                self.sys.renderer.draw_hexagon(keru_draw::Hexagon {
                    center: [cx, cy],
                    size: actual_size,
                    rotation: *rotation,
                    fill,
                    stroke_thickness: border_thickness,
                    x_clip,
                    y_clip,
                    texture,
                });
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

        self.sys.renderer.text.clear_changes();
        self.sys.changes.need_rerender = false;
    }

    /// Convenience function that creates a render pass, renders into it, and presents to the screen.
    ///
    /// Panics if the current surface texture can't be obtained from `surface`.
    pub fn autorender(&mut self, surface: &wgpu::Surface, background_color: wgpu::Color) {
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

    /// Draw a box with a gradient for custom rendering.
    ///
    /// This is useful when implementing custom rendering in the render plan.
    /// The box will be added to the current frame's render data.
    pub fn draw_box_gradient(
        &mut self,
        top_left: [f32; 2],
        size: [f32; 2],
        corner_radius: f32,
        rounded_corners: keru_draw::RoundedCorners,
        border_thickness: f32,
        start_color: [f32; 4],
        end_color: [f32; 4],
        gradient_angle: f32,
        clip_x: [f32; 2],
        clip_y: [f32; 2],
    ) {
        self.sys.renderer.draw_box(keru_draw::Box {
            top_left,
            size,
            corner_radius,
            rounded_corners,
            border_thickness,
            fill: keru_draw::Fill::Gradient {
                color_start: start_color,
                color_end: end_color,
                gradient_type: keru_draw::GradientType::Linear,
                angle: gradient_angle,
            },
            x_clip: clip_x,
            y_clip: clip_y,
            texture: None,
        });
    }

    /// Draw a solid color box for custom rendering.
    ///
    /// This is useful when implementing custom rendering in the render plan.
    /// The box will be added to the current frame's render data.
    pub fn draw_box(
        &mut self,
        top_left: [f32; 2],
        size: [f32; 2],
        corner_radius: f32,
        rounded_corners: keru_draw::RoundedCorners,
        border_thickness: f32,
        color: [f32; 4],
        clip_x: [f32; 2],
        clip_y: [f32; 2],
    ) {
        self.sys.renderer.draw_box(keru_draw::Box {
            top_left,
            size,
            corner_radius,
            rounded_corners,
            border_thickness,
            fill: keru_draw::Fill::Solid(color),
            x_clip: clip_x,
            y_clip: clip_y,
            texture: None,
        });
    }

    /// Draw a circle for custom rendering.
    ///
    /// This is useful when implementing custom rendering in the render plan.
    /// The circle will be added to the current frame's render data.
    pub fn draw_circle(
        &mut self,
        center: [f32; 2],
        radius: f32,
        color: [f32; 4],
        clip_x: [f32; 2],
        clip_y: [f32; 2],
    ) {
        self.sys.renderer.draw_circle(keru_draw::Circle {
            center,
            radius,
            fill: keru_draw::Fill::Solid(color),
            x_clip: clip_x,
            y_clip: clip_y,
            texture: None,
        });
    }
}

#[derive(Clone, Debug)]
pub enum ImageRef {
    Raster(LoadedImage),
    Svg(LoadedImage),
}
