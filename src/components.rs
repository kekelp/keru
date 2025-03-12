use crate as keru;
use keru::*;
use keru::Size::*;
use keru::Position::*;

#[derive(PartialEq, Eq)]
pub struct Tab(pub &'static str);

impl Ui {
    /// A component for vertical tabs
    #[track_caller]
    pub fn vertical_tabs(&mut self, tabs: &[Tab], current_tab: &mut usize) -> UiParent {
        #[node_key] const VERTICAL_TABS_TAB_BUTTON: NodeKey;

        self.subtree().start(|| {
            let max_n = tabs.len() - 1;
            if *current_tab >= max_n {
                *current_tab = max_n;
            }

            // Update the state in response to button clicks or keyboard presses
            for (i, _) in tabs.iter().enumerate() {
                if self.is_clicked(VERTICAL_TABS_TAB_BUTTON.sibling(i)) {
                    *current_tab = i;
                }
            }
            // todo: focused?
            let ilen = tabs.len() as isize;
            if self
                .key_input()
                .key_pressed_or_repeated(&winit::keyboard::Key::Named(
                    winit::keyboard::NamedKey::Tab,
                ))
            {
                if self.key_mods().shift_key() {
                    *current_tab = (((*current_tab as isize) - 1 + ilen) % ilen) as usize;
                } else {
                    *current_tab = (*current_tab + 1) % tabs.len();
                }
            }

            let h_stack = H_STACK.stack_spacing(0);
            let tabs_v_stack = V_STACK.size_x(Size::Pixels(250));
            let inactive_tab = BUTTON
                .corners(RoundedCorners::LEFT)
                .size_x(Size::Fill)
                .colors(self.theme().muted_background);
            let active_tab = inactive_tab.colors(self.theme().background);

            #[node_key] const VERTICAL_TABS_CONTENT_PANEL: NodeKey;
            let content_panel = PANEL
                .size_symm(Size::Fill)
                .colors(self.theme().background)
                .key(VERTICAL_TABS_CONTENT_PANEL);

            self.add(h_stack).nest(|| {
                self.add(tabs_v_stack).nest(|| {
                    for (i, name) in tabs.iter().enumerate() {
                        let key_i = VERTICAL_TABS_TAB_BUTTON.sibling(i);
                        let active = i == *current_tab;
                        let tab = if active { active_tab } else { inactive_tab };
                        let tab = tab.static_text(&name.0).key(key_i);
                        self.add(tab);
                    }
                });

                let content_nest = self.add(content_panel);

                return content_nest;
            })
        })
    }

    #[track_caller]
    pub fn slider(&mut self, value: &mut f32, min: f32, max: f32) {
        self.subtree().start(|| {
            let mut new_value = *value;
            if let Some(drag) = self.is_dragged(SLIDER_CONTAINER) {
                new_value += drag.relative_delta.x as f32 * (min - max);
            }

            if let Some(drag) = self.is_dragged(SLIDER_FILL) {
                new_value += drag.relative_delta.x as f32 * (min - max);
            }

            if new_value.is_finite() {
                new_value = new_value.clamp(min, max);
                *value = new_value;
            }

            let filled_frac = (*value - min) / (max - min);

            #[node_key] const SLIDER_CONTAINER: NodeKey;
            let slider_container = PANEL
                .size_x(Size::Fill)
                .size_y(Size::Pixels(45))
                .sense_drag(true)
                .key(SLIDER_CONTAINER);
            
            #[node_key] const SLIDER_FILL: NodeKey;
            let slider_fill = PANEL
                .size_y(Fill)
                .size_x(Size::Frac(filled_frac))
                .color(Color::KERU_RED)
                .position_x(Start)
                .padding_x(1)
                .sense_drag(true)
                .key(SLIDER_FILL);

            // todo: don't allocate here
            let text = format!("{:.2}", value);

            self.add(slider_container).nest(|| {
                self.add(slider_fill);
                self.text_line(&text);
            });
        });
    }

    #[track_caller]
    pub fn classic_slider(&mut self, value: &mut f32, min: f32, max: f32) {
        self.subtree().start(|| {
            // todo: combined with the handle's manual positioning, this is pretty awful. it means that the handle is drawn at zero in the first frame.
            // Currently, it relies on the anti-state tearing stuff to not stay at zero.
            // It should be fixed by making it's possible to express the " - handle_radius" part when using a Frac.
            let slider_width = match self.get_node(TRACK) {
                Some(track) => track.inner_size().x as f32,
                // this is just for the first frame. awkward.
                None => 1.0,
            };
            
            let handle_radius = 10.0;
            
            if let Some(click) = self.clicked_at(HITBOX) {
                *value = min + click.relative_position.x as f32 * max;
            }
            if let Some(drag) = self.is_dragged(HITBOX) {
                *value = min + drag.relative_position.x as f32 * max;
            }
        
            *value = value.clamp(min, max);
            
            let handle_position_frac = (*value - min) / (max - min);
            
            #[node_key] const TRACK: NodeKey;
            let slider_track = PANEL
                .size_x(Size::Fill)
                .size_y(Size::Pixels(10))
                .padding(0)
                .color(Color::GREY)
                .absorbs_clicks(false)
                .key(TRACK);
            
            #[node_key] const FILLED: NodeKey;
            let slider_filled = PANEL
                .size_y(Size::Pixels(14))
                .size_x(Size::Frac(handle_position_frac))
                .color(Color::KERU_RED)
                .position_x(Start)
                .padding_x(0)
                .absorbs_clicks(false)
                .key(FILLED);
            
            #[node_key] const HANDLE: NodeKey;
            let slider_handle = PANEL
                .size_x(Size::Pixels((handle_radius * 2.0) as u32))
                .size_y(Size::Pixels((handle_radius * 2.0) as u32))
                .color(Color::WHITE)
                // .position_x(Position::Static(Len::Frac(handle_position_frac)))
                .position_x(Position::Static(Len::Pixels((handle_position_frac * slider_width - handle_radius) as u32)))
                .position_y(Position::Center)
                .shape(Shape::Circle)
                .padding_x(0)
                .absorbs_clicks(false)
                .key(HANDLE);
            
            #[node_key] const SLIDER_CONTAINER: NodeKey;
            let slider_container = CONTAINER
                .size_x(Size::Fill)
                .size_y(Size::Pixels(45))
                .key(SLIDER_CONTAINER);
            
            #[node_key] const HITBOX: NodeKey;
            let hitbox = CONTAINER
                .size_x(Size::Fill)
                .size_y(Size::Pixels(30))
                .sense_click(true)
                .sense_drag(true)
                .padding(0)
                .key(HITBOX);
            
            self.add(slider_container).nest(|| {
                self.add(hitbox).nest(|| {
                    self.add(slider_track).nest(|| {
                        self.add(slider_filled);
                        self.add(slider_handle);
                    });
                });
            });
            
            self.format_scratch.clear();
        });
    }
}
