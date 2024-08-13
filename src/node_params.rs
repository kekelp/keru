use crate::{Arrange, Axis, Color, Interact, Layout, Len, NodeParams, Text, Position, Rect, Size, Stack, Xy};

pub const DEBUG_RED: Color = Color::rgba(1.0, 0.0, 0.0, 0.3);

use Size::*;
use Position::*;

pub(crate) const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    stack: None,
    text: None,
    rect: Rect {
        visible_rect: false,
        filled: false,
        color: Color::TRANSPARENT
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(Fraction(1.0)),
        position: Xy::new_symm(Start),
        padding: Xy::new_symm(Len::ZERO),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "ROOT",
};


// pub const DEFAULT: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "DEFAULT",
//     default_text: Some("Default"),
//     clickable: false,
//     visible_rect: true,
//     color: Color::BLUE,
//     size: Xy::new_symm(Fraction(1.0)),
//     position: Xy::new_symm(Center),
//     stack: None,
//     editable: false,
//     filled: true,
//     padding: Xy::new_symm(Len::ZERO),
// };

// pub const V_STACK: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "VStack",
//     default_text: None,
//     clickable: true,
//     visible_rect: false,
//     color: DEBUG_RED,
//     size: Xy::new(Size::FitToChildren, Size::FitToChildren),
//     position: Xy::new_symm(Center),
//     stack: Some(Stack {
//         arrange: Arrange::Start,
//         axis: Axis::Y,
//         spacing: Len::Pixels(5),
//     }),
//     editable: false,
//     filled: false,
//     padding: Xy::new_symm(Len::ZERO),
// };
// pub const H_STACK: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "HStack",
//     default_text: None,
//     visible_rect: false,
//     clickable: false,
//     color: DEBUG_RED,
//     size: Xy::new(Size::FitToChildren, Size::FitToChildren),
//     position: Xy::new_symm(Center),
//     stack: Some(Stack {
//         arrange: Arrange::Start,
//         axis: Axis::X,
//         spacing: Len::Pixels(5),
//     }),
//     editable: false,
//     filled: false,
//     padding: Xy::new_symm(Len::ZERO),
// };
// pub const MARGIN: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "Margin",
//     default_text: None,
//     clickable: false,
//     visible_rect: false,
//     color: DEBUG_RED,
//     size: Xy::new_symm(Fraction(0.9)),
//     position: Xy::new_symm(Center),
//     stack: None,
//     editable: false,
//     filled: false,
//     padding: Xy::new_symm(Len::ZERO),
// };

// pub const BUTTON: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "Button",
//     default_text: None,
//     clickable: true,
//     visible_rect: true,
//     color: Color::rgba(0.0, 0.1, 0.1, 0.9),
//     size: Xy::new_symm(TextContent),
//     position: Xy::new_symm(Center),
//     stack: None,
//     editable: false,
//     filled: true,
//     padding: Xy::new_symm(Len::Pixels(10)),
// };

// pub const LABEL: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "Label",
//     default_text: Some("Label"),
//     clickable: false,
//     visible_rect: true,
//     color: Color::BLUE,
//     size: Xy::new_symm(TextContent),
//     position: Xy::new_symm(Center),
//     stack: None,
//     editable: false,
//     filled: true,
//     padding: Xy::new_symm(Len::Pixels(10)),
// };

// pub const TEXT: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "Text",
//     default_text: Some("Text"),
//     clickable: false,
//     visible_rect: false,
//     color: Color::RED,
//     size: Xy::new_symm(TextContent),
//     position: Xy::new_symm(Center),
//     stack: None,
//     editable: false,
//     filled: false,
//     padding: Xy::new_symm(Len::Pixels(2)),
// };

// pub const TEXT_INPUT: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "Text input",
//     default_text: None,
//     clickable: true,
//     visible_rect: true,
//     color: Color::rgba(0.1, 0.0, 0.1, 0.9),
//     size: Xy::new_symm(TextContent),
//     position: Xy::new_symm(Center),
//     stack: None,
//     editable: true,
//     filled: true,
//     padding: Xy::new_symm(Len::Pixels(5)),
// };

// pub const PANEL: NodeParams = NodeParams {
//     #[cfg(debug_assertions)]
//     debug_name: "Panel",
//     default_text: None,
//     clickable: false,
//     visible_rect: true,
//     color: Color::rgba(0.1, 0.0, 0.1, 0.9),
//     size: Xy::new_symm(FitToChildren),
//     position: Xy::new_symm(Center),
//     stack: None,
//     editable: false,
//     filled: true,
//     padding: Xy::new_symm(Len::Pixels(10)),
// };

pub const DEFAULT: NodeParams = NodeParams {
    stack: None,
    text: Some(Text {
        default_text: "Default",
        editable: false,
    }),
    rect: Rect {
        visible_rect: true,
        filled: true,
        color: Color::BLUE,
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(Fraction(1.0)),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    
    #[cfg(debug_assertions)]
    debug_name: "DEFAULT",
};

pub const V_STACK: NodeParams = NodeParams {
    stack: Some(Stack {
        arrange: Arrange::Start,
        axis: Axis::Y,
        spacing: Len::Pixels(0),
    }),
    text: None,
    rect: Rect {
        visible_rect: false,
        filled: false,
        color: DEBUG_RED,
    },
    interact: Interact {
        clickable: true,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(20)),
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
        visible_rect: false,
        filled: false,
        color: DEBUG_RED,
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
        visible_rect: false,
        filled: false,
        color: DEBUG_RED,
    },
    interact: Interact {
        clickable: false,
    },
    layout: Layout {
        size: Xy::new_symm(Fraction(0.9)),
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
        visible_rect: true,
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
        visible_rect: true,
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
        visible_rect: false,
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
        visible_rect: true,
        filled: true,
        color: Color::rgba(0.1, 0.0, 0.1, 0.9),
    },
    interact: Interact {
        clickable: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
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
        visible_rect: true,
        filled: true,
        color: Color::rgba(0.1, 0.0, 0.1, 0.9),
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
    debug_name: "Panel",
};
