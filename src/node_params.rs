use crate::{Arrange, Axis, Color, Len, NodeParams, Position, Size, Stack, Xy};

pub const DEBUG_RED: Color = Color::rgba(1.0, 0.0, 0.0, 0.3);

use Size::*;
use Position::*;

pub(crate) const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "ROOT",
    static_text: None,
    visible_rect: false,
    clickable: false,
    color: Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.0,
    },
    size: Xy::new_symm(Fraction(1.0)),
    position: Xy::new_symm(Start),
    stack: None,
    editable: false,
    filled: true,
    padding: Xy::new_symm(Len::ZERO),
};


pub const DEFAULT: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "DEFAULT",
    static_text: Some("Default"),
    clickable: false,
    visible_rect: true,
    color: Color::BLUE,
    size: Xy::new_symm(Fraction(1.0)),
    position: Xy::new_symm(Center),
    stack: None,
    editable: false,
    filled: true,
    padding: Xy::new_symm(Len::ZERO),
};

pub const V_STACK: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "VStack",
    static_text: None,
    clickable: true,
    visible_rect: false,
    color: DEBUG_RED,
    size: Xy::new(Size::FitToChildren, Size::FitToChildren),
    position: Xy::new_symm(Center),
    stack: Some(Stack {
        arrange: Arrange::Start,
        axis: Axis::Y,
        spacing: Len::Pixels(5),
    }),
    editable: false,
    filled: false,
    padding: Xy::new_symm(Len::ZERO),
};
pub const H_STACK: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "HStack",
    static_text: None,
    visible_rect: false,
    clickable: false,
    color: DEBUG_RED,
    size: Xy::new(Size::FitToChildren, Size::FitToChildren),
    position: Xy::new_symm(Center),
    stack: Some(Stack {
        arrange: Arrange::Start,
        axis: Axis::X,
        spacing: Len::Pixels(5),
    }),
    editable: false,
    filled: false,
    padding: Xy::new_symm(Len::ZERO),
};
pub const MARGIN: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Margin",
    static_text: None,
    clickable: false,
    visible_rect: false,
    color: DEBUG_RED,
    size: Xy::new_symm(Fraction(0.9)),
    position: Xy::new_symm(Center),
    stack: None,
    editable: false,
    filled: false,
    padding: Xy::new_symm(Len::ZERO),
};

pub const BUTTON: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Button",
    static_text: None,
    clickable: true,
    visible_rect: true,
    color: Color::rgba(0.0, 0.1, 0.1, 0.9),
    size: Xy::new_symm(TextContent),
    position: Xy::new_symm(Center),
    stack: None,
    editable: false,
    filled: true,
    padding: Xy::new_symm(Len::Pixels(10)),
};

pub const LABEL: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Label",
    static_text: Some("Label"),
    clickable: false,
    visible_rect: true,
    color: Color::BLUE,
    size: Xy::new_symm(TextContent),
    position: Xy::new_symm(Center),
    stack: None,
    editable: false,
    filled: true,
    padding: Xy::new_symm(Len::Pixels(10)),
};

pub const TEXT: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Text",
    static_text: Some("Text"),
    clickable: false,
    visible_rect: false,
    color: Color::RED,
    size: Xy::new_symm(TextContent),
    position: Xy::new_symm(Center),
    stack: None,
    editable: false,
    filled: false,
    padding: Xy::new_symm(Len::Pixels(2)),
};

pub const TEXT_INPUT: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Text input",
    static_text: None,
    clickable: true,
    visible_rect: true,
    color: Color::rgba(0.1, 0.0, 0.1, 0.9),
    size: Xy::new_symm(TextContent),
    position: Xy::new_symm(Center),
    stack: None,
    editable: true,
    filled: true,
    padding: Xy::new_symm(Len::Pixels(5)),
};

pub const PANEL: NodeParams = NodeParams {
    #[cfg(debug_assertions)]
    debug_name: "Panel",
    static_text: None,
    clickable: false,
    visible_rect: true,
    color: Color::rgba(0.1, 0.0, 0.1, 0.9),
    size: Xy::new_symm(FitToChildren),
    position: Xy::new_symm(Center),
    stack: None,
    editable: false,
    filled: true,
    padding: Xy::new_symm(Len::Pixels(10)),
};