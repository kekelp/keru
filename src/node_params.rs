use crate::{Arrange, Axis, Color, NodeParams, Position, Size, Stack, Xy};

pub const DEBUG_RED: Color = Color::rgba(1.0, 0.0, 0.0, 0.3);

pub const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "ROOT",
    text: None,
    visible_rect: false,
    clickable: false,
    color: Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.0,
    },
    size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Start),
    is_stack: None,
    editable: false,
    filled: true,
};


pub const DEFAULT: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "DEFAULT",
    text: Some("Default"),
    clickable: false,
    visible_rect: true,
    color: Color::BLUE,
    size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Center),
    is_stack: None,
    editable: false,
    filled: true,
};

pub const V_STACK: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "VStack",
    text: None,
    clickable: true,
    visible_rect: false,
    color: DEBUG_RED,
    size: Xy::new(Size::PercentOfAvailable(1.0), Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Center),
    is_stack: Some(Stack {
        arrange: Arrange::Start,
        axis: Axis::Y,
    }),
    editable: false,
    filled: false,
};
pub const H_STACK: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "HStack",
    text: None,
    visible_rect: false,
    clickable: false,
    color: DEBUG_RED,
    size: Xy::new(Size::PercentOfAvailable(1.0), Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Center),
    is_stack: Some(Stack {
        arrange: Arrange::End,
        axis: Axis::X,
    }),
    editable: false,
    filled: false,
};
pub const MARGIN: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Margin",
    text: None,
    clickable: false,
    visible_rect: false,
    color: DEBUG_RED,
    size: Xy::new_symm(Size::PercentOfAvailable(0.9)),
    position: Xy::new_symm(Position::Center),
    is_stack: None,
    editable: false,
    filled: false,
};

pub const BUTTON: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Button",
    text: None,
    clickable: true,
    visible_rect: true,
    color: Color::rgba(0.0, 0.1, 0.1, 0.9),
    size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Center),
    is_stack: None,
    editable: false,
    filled: true,
};

pub const LABEL: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Label",
    text: Some("Label"),
    clickable: false,
    visible_rect: true,
    color: Color::BLUE,
    size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Center),
    is_stack: None,
    editable: false,
    filled: true,
};

pub const TEXT: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Text",
    text: Some("Text"),
    clickable: false,
    visible_rect: false,
    color: Color::RED,
    size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Center),
    is_stack: None,
    editable: false,
    filled: false,
};

pub const TEXT_INPUT: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Text input",
    text: None,
    clickable: true,
    visible_rect: true,
    color: Color::rgba(0.1, 0.0, 0.1, 0.9),
    size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Center),
    is_stack: None,
    editable: true,
    filled: true,
};

pub const PANEL: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Panel",
    text: None,
    clickable: false,
    visible_rect: true,
    color: Color::rgba(0.1, 0.0, 0.1, 0.9),
    size: Xy::new_symm(Size::PercentOfAvailable(1.0)),
    position: Xy::new_symm(Position::Center),
    is_stack: None,
    editable: false,
    filled: true,
};