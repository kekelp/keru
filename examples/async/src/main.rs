use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl State {
    pub fn update_ui(&mut self) {
        match &mut self.file_state {
            FileState::NotStarted => {
                if self.ui.add(BUTTON.static_text("Click to load the file")).is_clicked(&mut self.ui) {
                    self.start_loading_file();
                }
            }
            FileState::Loading(file_future) => {
                let mut context = TaskContext::from_waker(&self.ui_waker);
                match file_future.as_mut().poll(&mut context) {
                    Poll::Pending => {
                        // If we had a complex async function that can pause at multiple times, it would always be just "Pending" regardless of where it's currently stuck at.
                        // If we were building state machines by hand, we could have more descriptive messages differentiating the current state of the operation. 
                        self.ui.add(LABEL.static_text("Loading..."));
                    },
                    Poll::Ready(file_content) => {
                        self.file_state = FileState::Loaded(file_content);
                    }
                };
            }
            FileState::Loaded(content) => {
                self.ui.add(LABEL.text(&content));
            }
        }
    }

    fn start_loading_file(&mut self) {
        let future = Box::pin(async {
            async_std::task::sleep(std::time::Duration::from_secs(1)).await;
            let mut file = async_std::fs::File::open("src/ui.rs").await.unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).await.unwrap();
            contents
        });
        self.file_state = FileState::Loading(future);
    }
}

fn main() {
    // basic_env_logger_init();
    let mut state = State::default();
    state.show = true;
    run_example_loop(state, State::update_ui);
}
