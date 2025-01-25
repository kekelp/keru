use crate::*;

pub fn reactive<T>(variables_changed: bool, reactive_block: impl FnOnce() -> T) -> T {
    if ! variables_changed {
        thread_local::push_skip_block();
    }

    let block_result = reactive_block();

    if ! variables_changed {
        thread_local::pop_skip_block();
    }

    return block_result;
}

pub fn can_skip() -> bool {
    return thread_local::THREAD_STACKS.with(|stack| {
        return stack.borrow_mut().reactive > 0;
    });
}