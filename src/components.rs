use crate as keru;
use keru::*;

impl Ui {
    pub fn vertical_tabs(&mut self, tabs: &[&str], current_tab: &mut usize) -> UiParent {
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
            let tabs_v_stack = V_STACK.size_x(Size::Frac(0.12));
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
                        let tab = tab.text(name).key(key_i);
                        self.add(tab);
                    }
                });

                let content_nest = self.add(content_panel);

                return content_nest;
            })
        })
    }
}
