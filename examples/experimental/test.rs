//! What still depends on a text box's own height, now that `update_text_boxes` doesn't set it.
//!
//! With the `set_size` call commented out, `TextBox::height` stays at the 500.0 it was created
//! with in `set_params_text` (node.rs), forever. Almost nothing notices, because keru overrides
//! every place the box's own rect would otherwise be used:
//!
//!   - `hit_full_rect` and `compute_hit_test_shape` take the `explicit_hitbox` branch, and
//!     `set_hitbox` still runs every frame from the *animated* rect
//!   - `effective_clip_rect` and the scroll-offset clip rect are behind `auto_clip`, which keru
//!     never turns on; it sets an explicit screen-space clip rect instead
//!
//! The exception is cross-box selection. keru links every selectable text box into a chain each
//! frame (node.rs), and the handoff between links goes through `is_cursor_past_end`:
//!
//! ```ignore
//! let effective_height = self.height.min(self.layout.height());
//! if local_pos.y > effective_height { return true; }
//! ```
//!
//! So the threshold at which a downward drag leaves one box and enters the next is now
//! `min(500.0, text_height)` instead of `min(node_height, text_height)`.
//!
//! For the first and third paragraphs those are the same number, and everything behaves. The
//! middle one separates them: its node is 40px tall and clips, but its text is several lines, so
//! the old threshold was the visible bottom edge and the new one is the true bottom of the text,
//! well below the clip and invisible.
//!
//! # What to try
//!
//! - Drag a selection from the first paragraph downwards, through the clipped one, into the last.
//!   Watch where the selection moves on: it used to hand off at the visible edge of the clipped
//!   box, and should now hang there until the cursor is past where the hidden text ends.
//! - Hold the drag and press the button so the stack animates under the cursor. The hitboxes
//!   follow the animation (they're rebuilt every frame from the animated rect), so hit testing
//!   stays correct — it's only the handoff threshold that's frozen.
//! - Animations run at 0.15x so this is slow enough to watch.

use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub extra: bool,
}

const PARA_1: &str = "First paragraph. Its node is as tall as its text, so the old threshold and \
    the new one are the same number, and selection behaves exactly as it used to.";

const CLIPPED: &str = "Second paragraph, in a node that is 40 pixels tall and clips. Its text is \
    much taller than that, so most of these lines aren't visible at all. This is the one case \
    where the box's own height and the height of its text disagree, which is exactly what \
    is_cursor_past_end compares the cursor against.";

const PARA_3: &str = "Third paragraph, back to a normal node. Selection should arrive here once \
    the cursor is past the end of the second one.";

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        ui.set_global_animation_speed(0.15);

        #[node_key] const TOGGLE: NodeKey;

        if ui.is_clicked(TOGGLE) {
            self.extra = !self.extra;
        }

        let paragraph = TEXT_PARAGRAPH
            .size(Size::Fill, Size::FitContent)
            .text_size(16.0)
            .text_alignment(keru_draw::parley::Alignment::Start);

        ui.add(PANEL.size(Size::Pixels(520.0), Size::FitContent)).nest(|| {
            ui.add(V_STACK.size(Size::Fill, Size::FitContent).animate_layout(true)).nest(|| {

                ui.add(BUTTON.text("Toggle a paragraph (animates the stack)").key(TOGGLE));

                ui.add(paragraph.text(PARA_1));

                // The interesting one: node much shorter than its text, and clipping.
                ui.add(paragraph
                    .size_y(Size::Pixels(40.0))
                    .clip_children_y(true)
                    .text(CLIPPED));

                if self.extra {
                    ui.add(paragraph.text("An extra paragraph, to make the stack move."));
                }

                ui.add(paragraph.text(PARA_3));
            });
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
