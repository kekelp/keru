use crate as keru;
use keru::*;
use keru::node_library::*;

use crate::thread_future_2::{ThreadFuture, run_in_background};
use std::sync::Arc;
use std::task::Poll;

pub struct AsyncButton<T>
where T: Send + 'static {
    async_function: Arc<dyn Fn() -> T + Send + Sync + 'static>,
    idle_text: &'static str,
    loading_text: &'static str,
    key: Option<ComponentKey<Self>>,
}

impl<T> AsyncButton<T>
where T: Send + 'static {
    pub fn new<F>(function: F, idle_text: &'static str, loading_text: &'static str) -> Self
    where F: Fn() -> T + Send + Sync + 'static {
        Self {
            async_function: Arc::new(function),
            idle_text,
            loading_text,
            key: None,
        }
    }

    pub fn key(mut self, key: ComponentKey<Self>) -> Self {
        self.key = Some(key);
        self
    }
}

impl<T> Component for AsyncButton<T>
where T: Send + Sync + 'static {
    type AddResult = Poll<T>;
    type ComponentOutput = ();
    type State = Option<ThreadFuture<T>>;

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Option<ThreadFuture<T>>) -> Self::AddResult {
        #[node_key] const ASYNC_BUTTON: NodeKey;

        let clickable: bool;
        let button_text: &'static str;
        let result: Poll<T>;

        match state.as_ref().map(|f| f.poll()) {
            None => {
                button_text = self.idle_text;
                clickable = true;
                result = Poll::Pending;
            }
            Some(Poll::Pending) => {
                button_text = self.loading_text;
                clickable = false;
                result = Poll::Pending;
            }
            Some(Poll::Ready(val)) => {
                button_text = self.idle_text;
                clickable = true;
                result = Poll::Ready(val);
                *state = None; // Reset to idle so we can restart
            }
        };

        let button = BUTTON.static_text(button_text).key(ASYNC_BUTTON);

        ui.add(button);

        if clickable && ui.is_clicked(ASYNC_BUTTON) {
            let waker = ui.ui_waker_safe();
            let func = Arc::clone(&self.async_function);
            *state = Some(run_in_background(
                move || func(),
                move || waker.set_update_needed(),
            ));
        }

        return result;
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        self.key
    }
}
