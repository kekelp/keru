#![allow(unused)]
use keru::*;
use keru::example_window_loop::*;

// Tests that readd_branch handles exiting nodes correctly.
//
// When a node inside a branch is removed, it starts an exit animation and stays
// in the parent's child list with exiting=true. If readd_branch is then called
// for that parent, it must unlink the exiting node before cleanup_and_stuff
// re-adds it — otherwise cleanup_and_stuff would append it after itself,
// creating a self-cycle in the linked list.
//
// To exercise this: toggle the extra panel off. The first frame rebuilds the
// branch normally (extra is omitted → starts exiting). Every frame after that,
// readd_branch is called while extra is still exiting. If the list gets
// corrupted, layout/render will hang or crash.

struct State {
    show_extra: bool,
    changed: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const BRANCH_ROOT: NodeKey;
    #[node_key] const TOGGLE: NodeKey;
    #[node_key] const EXTRA: NodeKey;

    let toggle = BUTTON
        .static_text(&"Toggle extra panel")
        .key(TOGGLE);

    let extra = PANEL
        .color(Color::KERU_GREEN)
        .size_symm(Size::Pixels(100.0))
        .exit_slide(SlideEdge::Top, SlideDirection::Out)
        .key(EXTRA);

    if ui.is_clicked(TOGGLE) {
        state.show_extra = !state.show_extra;
        state.changed = true;
    }

    let branch = V_STACK.key(BRANCH_ROOT);

    if state.changed {
        ui.add(branch).nest(|| {
            ui.add(toggle);
            if state.show_extra {
                ui.add(extra);
            }
        });
        state.changed = false;
    } else {
        ui.readd_branch(BRANCH_ROOT);
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();

    let state = State {
        show_extra: true,
        changed: true,
    };
    example_window_loop::run_example_loop(state, update_ui);
}
