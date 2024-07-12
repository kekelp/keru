use crate::canvas::{Canvas, EpicRotation};
use glam::dvec2;
use crate::helper::*;

// this is needed to fix some crap with macros: https://github.com/rust-lang/rust/pull/52234#issuecomment-894851497
// when ui will be in its own crate, this won't happen anymore
use crate::*;
use crate::ui::*;
use crate::ui::Axis::Y;
use view_derive::derive_view;
use winit::{
    event::{Event, MouseButton}, event_loop::EventLoopWindowTarget, keyboard::KeyCode
};

impl State {

    pub fn update_canvas(&mut self) {
        self.canvas.draw_dots();

        self.zoom();
        self.rotate_and_pan();

        if self.canvas.end_stroke {
            self.canvas.mouse_dots.clear();
            self.canvas.end_stroke = false;
        }

        if self.canvas.need_backup {
            self.canvas.push_backup();
            self.canvas.need_backup = false;
        }

    }

    pub fn zoom(&mut self) {
        // todo, might be better to keep the last mouse pos *before the scrolling started*
        let mouse_before = self.canvas.screen_to_image(self.canvas.last_mouse_pos.x, self.canvas.last_mouse_pos.y);
        let mouse_before = dvec2(mouse_before.0, mouse_before.1);

        let (_x, y) = self.ctx.input.scroll_diff();

        let min_zoom = 0.01;
        let max_zoom = 1000.0;
        let delta = y as f64 * 0.4;

        let curve_factor = 0.3 * ((0.01 + self.canvas.scale.x).powf(1.1) - 0.01).abs();

        let new_val = self.canvas.scale.x + delta * curve_factor;

        if new_val > min_zoom && new_val < max_zoom && ! new_val.is_infinite() && ! new_val.is_nan() {
            self.canvas.scale = dvec2(new_val, new_val);
        }

        let mouse_after = self.canvas.screen_to_image(self.canvas.last_mouse_pos.x, self.canvas.last_mouse_pos.y);
        let mouse_after = dvec2(mouse_after.0, mouse_after.1);

        let diff = mouse_after - mouse_before;
        
        // convert the mouse position diff (screen space) to image space.
        // --> only rotation and y invert
        let diff = dvec2(diff.x, -diff.y);
        let huh = self.canvas.rotation.inverse_vec();
        let diff = diff.rotate(huh);

        self.canvas.translation += diff;


        self.canvas.update_shader_transform(&self.ctx.queue);
    }

    pub fn rotate_and_pan(&mut self) -> Option<()> {
        let pan = (self.ctx.input.key_held(KeyCode::Space) && self.ctx.input.mouse_held(MouseButton::Left)) 
        || self.ctx.input.mouse_held(MouseButton::Middle);

        if pan {

            let (x, y) = self.ctx.input.cursor_diff();
            let delta = dvec2(x as f64, y as f64);
            if self.ctx.input.held_shift() {

                let before = self.ctx.input.cursor()?;
                
                let before = dvec2(before.0 as f64, before.1 as f64);

                // todo, I think in some cases it should be centered around image coordinates.
                // for example when the whole image is zoomed out and it's in the right half of the viewport.
                let before = self.canvas.center_screen_coords(before);
                
                let after = before + delta;


                let angle = after.angle_to(before);

                let new_angle = self.canvas.rotation.angle() + angle;
                self.canvas.rotation = EpicRotation::new(new_angle);


            } else {

                self.canvas.translation += delta / self.canvas.scale;

                self.canvas.update_shader_transform(&self.ctx.queue);
            }

        }

        return Some(());
    }
}