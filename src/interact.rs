use crate::{ui_time_f32, Id, Ui};

impl Ui {

    // called on every mouse movement AND on every frame.
    // todo: think if it's really worth it to do this on every mouse movement.
    pub fn resolve_hover(&mut self) {
        let topmost_mouse_hit = self.scan_mouse_hits();

        if let Some(hovered_id) = topmost_mouse_hit {
            self.sys.hovered.push(hovered_id);
            let t = ui_time_f32();
            let node = self.nodes.get_by_id(&hovered_id).unwrap();
            node.last_hover = t;
        }
    }

    pub fn resolve_click(&mut self) -> bool {
        let topmost_mouse_hit = self.scan_mouse_hits();

        // defocus when use clicking anywhere outside.
        self.sys.focused = None;

        if let Some(clicked_id) = topmost_mouse_hit {
            self.sys.waiting_for_click_release = true;

            self.sys.clicked.push(clicked_id);
            let t = ui_time_f32();
            let node = self.nodes.get_by_id(&clicked_id).unwrap();
            node.last_click = t;

            if let Some(_) = node.text_id {
                if let Some(text) = node.params.text_params{
                    if text.editable {
                        self.sys.focused = Some(clicked_id);
                    }
                }
            }

            if let Some(id) = node.text_id {
                let text_area = &mut self.sys.text.text_areas[id];
                let (x, y) = (
                    self.sys.part.mouse_pos.x - text_area.left,
                    self.sys.part.mouse_pos.y - text_area.top,
                );

                // todo: with how I'm misusing cosmic-text, this might become "unsafe" soon (as in, might be incorrect or cause panics, not actually unsafe).
                // I think in general, there should be a safe version of hit() that just forces a rerender just to be sure that the offset is safe to use.
                // But in this case, calling this in resolve_mouse_input() and not on every winit mouse event probably means it's safe

                // actually, the enlightened way is that cosmic_text exposes an "unsafe" hit(), but we only ever see the string + cursor + buffer struct, and call that hit(), which doesn't return an offset but just mutates the one inside.
                text_area.buffer.hit(x, y);
            }
        }

        let consumed = topmost_mouse_hit.is_some();
        return consumed;
    }

    pub fn resolve_click_release(&mut self) -> bool {
        self.sys.waiting_for_click_release = false;
        let topmost_mouse_hit = self.scan_mouse_hits();
        let consumed = topmost_mouse_hit.is_some();
        self.sys.clicked.clear();
        return consumed;
    }

    pub fn scan_mouse_hits(&mut self) -> Option<Id> {
        self.sys.mouse_hit_stack.clear();

        for rect in &self.sys.rects {
            if self.sys.part.mouse_hit_rect(rect) {
                self.sys.mouse_hit_stack.push((rect.id, rect.z));
            }
        }

        // only the one with the highest z is actually clicked.
        // in practice, nobody ever sets the Z. it depends on the order.
        let mut topmost_hit = None;

        let mut max_z = f32::MAX;
        for (id, z) in self.sys.mouse_hit_stack.iter().rev() {
            if *z < max_z {
                max_z = *z;
                topmost_hit = Some(*id);
            }
        }

        return topmost_hit;
    }
}