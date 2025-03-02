use std::fmt::Display;

use crate::*;

/// A struct referring to a reactive block created with [`Ui::reactive()`].
///  
/// Use [`start()`](Self::start()) to start the block.
pub struct Reactive {
    state_changed: bool,
}

impl Ui {
    /// Start a reactive block.
    /// 
    /// This function seems to work from the examples, but it's still kind of experimental. It might be reimplemented in a more robust way in the future.
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
    ///     let state_changed = ui.check_changes(score);
    ///     ui.reactive(state_changed).start(|| {
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
    /// 
    pub fn reactive(&mut self, state_changed: bool) -> Reactive {
        return Reactive {
            state_changed,
        };
    }
}

impl Reactive {
    /// Start the reactive block created with [`Ui::reactive()`].
    /// 
    /// This method should be called right after after calling [`Ui::reactive()`].
    pub fn start<T>(&mut self, reactive_block: impl FnOnce() -> T) -> T {
        if self.state_changed {
            log::trace!("Reactive block: state changed");
        } else {
            log::trace!("Reactive block: state unchanged");
        }
        
        if !self.state_changed {
            thread_local::push_skip_block();
        }
        
        let result = reactive_block();
        
        if !self.state_changed {
            thread_local::pop_skip_block();
        }
        
        return result;
    }
}

/// Returns `true` if currently in a reactive block that is being skipped.
/// 
/// This can be used to skip expensive computations that are only useful when the GUI actually updates, such as formatting complex values into strings.
pub fn is_in_skipped_reactive_block() -> bool {
    return thread_local::THREAD_STACKS.with(|stack| {
        return stack.borrow_mut().reactive > 0;
    });
}
