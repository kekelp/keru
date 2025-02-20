use crate::*;

/// Start a reactive block.
/// 
/// If `state_changed` is false, the [`Ui`] will assume that the GUI elements inside the block haven't changed either, and it will be able to skip hashing and diffing operations, improving performance.
/// 
/// To use this function correctly, you must be sure that all the GUI code inside the block depends only on a well known set of variables, and you must be able to determine if these variables changed since the last frame or not.
/// 
/// A good place to use this function is when writing self-contained "component" functions.
/// An easy way to keep track of whether variables have changed is to keep wrap them in an [`Observer`] struct, but there are many other valid strategies, depending on the context.
/// 
/// ```
/// # use keru::*;
/// fn display_score(ui: &mut Ui, score: &mut Observer<i32>) {
///     let state_changed = score.changed();
///     reactive(state_changed, || {
///         // as long as the GUI code inside here depends only on the value of `score`, this is correct.
///         ui.label(score);
///         // if it depended on something like the system's time,
///         // the reactive block would incorrectly skip updating it.
///     });    
/// }
/// ```
/// 
/// If code inside the reactive block changes the score, the GUI won't be updated until the following frame, as the value of `state_changed` is determined once at the start of the reactive block.
/// 
/// If you're trying to get your application to use less CPU when fully idle, reactive blocks are *not* the solution. You should set up your `winit` loop to go to sleep properly, instead. 
/// 
/// Reactive blocks are only useful in complex GUIs, to avoid running a full update on the whole visible GUI when only a part changed.
pub fn reactive<T>(state_changed: bool, reactive_block: impl FnOnce() -> T) -> T {
    if ! state_changed {
        thread_local::push_skip_block();
    }

    let block_result = reactive_block();

    if ! state_changed {
        thread_local::pop_skip_block();
    }

    return block_result;
}

/// Returns `true` if currently in a reactive block that is being skipped.
/// 
/// This can be used to skip expensive computations that are only useful when the GUI actually updates. For example, formatting values into strings.
pub fn can_skip() -> bool {
    return thread_local::THREAD_STACKS.with(|stack| {
        return stack.borrow_mut().reactive > 0;
    });
}
