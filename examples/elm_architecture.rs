/// This is a counter implemented with something similar to the Elm architecture.
/// 
/// This example is just meant to be a reminder that even if the library doesn't enforce any particular architecture, you are still the master of your own destiny. If you don't want to mutate your state in the same function where you build your GUI, you can just not do it.

use keru::*;
use keru::node_library::*;

pub struct Model {
    pub count: i32,
}

enum Message {
    Increase,
    Decrease,
}

fn update(model: &mut Model, message: Message) {
    match message {
        Message::Increase => model.count += 1,
        Message::Decrease => model.count -= 1,
    }
}

fn view(model: &Model, ui: &mut Ui) -> Option<Message> {
    #[node_key] const INCREASE: NodeKey;
    #[node_key] const DECREASE: NodeKey;

    let increase_button = BUTTON
        .color(Color::RED)
        .text("Increase")
        .key(INCREASE);

    let decrease_button = BUTTON
        .text("Decrease")
        .key(DECREASE);

    ui.add(V_STACK).nest(|| {
        ui.add(increase_button);
        ui.add(LABEL.text(&model.count.to_string()));
        ui.add(decrease_button);
    });

    if ui.is_clicked(INCREASE) {
        return Some(Message::Increase);
    }
    if ui.is_clicked(DECREASE) {
        return Some(Message::Decrease);
    }

    None
}

fn elm(model: &mut Model, ui: &mut Ui) {
    let message = view(model, ui);
    // We could also return a Vec<Message>, or store them into a queue somewhere.
    if let Some(message) = message {
        update(model, message);
    }
}

fn main() {
    let model = Model { count: 0 };
    example_window_loop::run_example_loop(model, elm);
}
