
use std::cell::RefCell;
use bumpalo::Bump;

thread_local! {
    /// Thread local bump arena for temporary allocations
    static THREAD_ARENA: RefCell<Bump> = RefCell::new(Bump::new());
}

/// Access keru's thread-local bump arena for temporary allocations.
/// Useful for small local allocations without passing an arena around, like formatting strings to show in the gui.
///
/// The arena is reset at the end of each frame, when [`Ui::finish_frame()`] is called.
/// 
/// This function is useful when implementing a reusable component with the [`Component`] traits, since you can't easily access all of your state from within the trait impl. In other cases, it might be more convenient to use your own arena.
///
/// # Panics
/// Panics if [`Ui::finish_frame()`] is called from inside the passes closure.
///
/// # Example
/// ```no_run
/// # use keru::*;
/// # let mut ui: Ui = unimplemented!();
/// # let float_value = 6.7;
/// with_arena(|a| {
///     let text = bumpalo::format!(in a, "{:.2}", float_value);
///     ui.add(LABEL.text(&text)); // Great
///     // ui.finish_frame(); // Don't do this.
/// });
/// 
/// ui.finish_frame(); // Now it's fine.
/// ```
pub fn with_arena<F, R>(f: F) -> R
where
    F: FnOnce(&Bump) -> R,
{
    THREAD_ARENA.with(|arena| {
        f(&arena.borrow())
    })
}

pub(crate) fn reset_arena() {
    THREAD_ARENA.with(|arena| {
        arena.borrow_mut().reset();
    });
}
