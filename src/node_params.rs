use crate::{Arrange, Axis, Color, Interact, Layout, Len, NodeParams, Text, Position, Rect, Size, Stack, Xy};
use Size::*;
use Position::*;

pub(crate) const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    stack: None,
    text: None,
    rect: Rect {
        visible: false,
        filled: false,
        color: Color::TRANSPARENT
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(Frac(1.0)),
        position: Xy::new_symm(Start),
        padding: Xy::new_symm(Len::ZERO),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "ROOT",
};

pub const DEFAULT: NodeParams = NodeParams {
    stack: None,
    text: Some(Text {
        default_text: "Default",
        editable: false,
    }),
    rect: Rect {
        visible: true,
        filled: true,
        color: Color::BLUE,
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(Frac(1.0)),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "DEFAULT",
};

pub const V_STACK: NodeParams = NodeParams {
    stack: Some(Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: Len::Pixels(10),
    }),
    text: None,
    rect: Rect {
        visible: false,
        filled: false,
        color: Color::DEBUG_RED,
    },
    interact: Interact {
        clickable: true,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "VStack",
};

pub const H_STACK: NodeParams = NodeParams {
    stack: Some(Stack {
        arrange: Arrange::Start,
        axis: Axis::X,
        spacing: Len::Pixels(5),
    }),
    text: None,
    rect: Rect {
        visible: false,
        filled: false,
        color: Color::DEBUG_RED,
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "HStack",
};

pub const MARGIN: NodeParams = NodeParams {
    stack: None,
    text: None,
    rect: Rect {
        visible: false,
        filled: false,
        color: Color::DEBUG_RED,
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(Frac(0.9)),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "Margin",
};

pub const BUTTON: NodeParams = NodeParams {
    stack: None,
    text: None,
    rect: Rect {
        visible: true,
        filled: true,
        color: Color::rgba(0.0, 0.1, 0.1, 0.9),
    },
    interact: Interact {
        clickable: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "Button",
};

pub const LABEL: NodeParams = NodeParams {
    stack: None,
    text: Some(Text {
        default_text: "Label",
        editable: false,
    }),
    rect: Rect {
        visible: true,
        filled: true,
        color: Color::BLUE,
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "Label",
};

pub const TEXT: NodeParams = NodeParams {
    stack: None,
    text: Some(Text {
        default_text: "Text",
        editable: false,
    }),
    rect: Rect {
        visible: false,
        filled: false,
        color: Color::RED,
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(2)),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "Text",
};

pub const TEXT_INPUT: NodeParams = NodeParams {
    stack: None,
    text: Some(Text {
        default_text: "",
        editable: true,
    }),
    rect: Rect {
        visible: true,
        filled: true,
        color: Color::rgba(0.1, 0.0, 0.1, 0.9),
    },
    interact: Interact {
        clickable: true,
    },
    layout: Layout {
        size: Xy::new_symm(Fill),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(5)),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "Text input",
};

pub const PANEL: NodeParams = NodeParams {
    stack: None,
    text: None,
    rect: Rect {
        visible: true,
        filled: true,
        color: Color::BLUE,
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(2)),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "Panel",
};
