use crate as keru;
use keru::*;
use keru::Size::*;
use keru::Position::*;

#[derive(PartialEq, Eq)]
pub struct Tab(pub &'static str);

impl Ui {
    /// A component for vertical tabs
    pub fn vertical_tabs(&mut self, tabs: &[Tab], current_tab: &mut usize) -> UiParent {
        #[node_key]
        const TAB_BUTTON: NodeKey;

        self.subtree().start(|| {
            let max_n = tabs.len() - 1;
            if *current_tab >= max_n {
                *current_tab = max_n;
            }

            // Update the state in response to button clicks or keyboard presses
            for (i, _) in tabs.iter().enumerate() {
                if self.is_clicked(TAB_BUTTON.sibling(i)) {
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
            let content_panel = PANEL.size_symm(Size::Fill).colors(self.theme().background);

            self.add(h_stack).nest(|| {
                self.add(tabs_v_stack).nest(|| {
                    for (i, name) in tabs.iter().enumerate() {
                        let key_i = TAB_BUTTON.sibling(i);
                        let active = i == *current_tab;
                        let tab = if active { active_tab } else { inactive_tab };
                        // we could ask for 'static strings so we can Static() here, but I doubt anybody cares  
                        let tab = tab.static_text(&name.0).key(key_i);
                        self.add(tab);
                    }
                });

                let content_nest = self.add(content_panel);

                return content_nest;
            })
        })
    }


    pub fn slider(&mut self, value: &mut f32, min: f32, max: f32) {
        let slider_height = match self.get_node(SLIDER_CONTAINER) {
            Some(container) => container.inner_size().x as f32,
            // this is just for the first frame. awkward.
            None => 1.0,
        };

        let (x, _) = self.is_dragged(SLIDER_CONTAINER);
        *value += (x as f32) / slider_height * (min - max);
        
        let (x, _) = self.is_dragged(SLIDER_FILL);
        *value += (x as f32) / slider_height * (min - max);

        *value = value.clamp(min, max);
        let filled_frac = (*value - min) / (max - min);

        #[node_key] const SLIDER_CONTAINER: NodeKey;
        let slider_container = PANEL
            .size_x(Size::Fill)
            .size_y(Size::Pixels(60))
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

        self.format_scratch.clear();
    }
}
