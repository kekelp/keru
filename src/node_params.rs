use crate::*;
use crate::color::*;
use std::hash::{Hash, Hasher};
use rustc_hash::FxHasher;

#[derive(Debug, Copy, Clone)]
pub struct NodeParams {
    pub text_params: Option<TextOptions>,
    pub stack: Option<Stack>,
    pub rect: Rect,
    pub interact: Interact,
    pub layout: Layout,
    pub key: NodeKey,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Fixed(Len),
    Fill,
    FitContent,
    FitContentOrMinimum(Len),
    AspectRatio(f32),
}

// Get a load of this dogshit that I have to write
impl Hash for Size {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Size::*;
        match self {
            Fixed(len) => (0u8, len).hash(state),
            Fill => 1u8.hash(state),
            FitContent => 2u8.hash(state),
            FitContentOrMinimum(len) => (3u8, len).hash(state),
            AspectRatio(ratio) => (4u8, ratio.to_bits()).hash(state),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Position {
    Center,
    Start,
    End,
    Static(Len),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Hash)]
pub struct Stack {
    pub arrange: Arrange,
    pub axis: Axis,
    pub spacing: Len,
}
impl Stack {
    pub const DEFAULT: Stack = Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: Len::Pixels(5),
    };
    pub const fn arrange(mut self, arrange: Arrange) -> Self {
        self.arrange = arrange;
        return self;
    }
    pub const fn spacing(mut self, spacing: Len) -> Self {
        self.spacing = spacing;
        return self;
    }
    pub const fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        return self;
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub enum Arrange {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

// might as well move to Rect? but maybe there's issues with non-clickable stuff absorbing the clicks.
#[derive(Debug, Copy, Clone, Hash)]
pub struct Interact {
    pub click_animation: bool,
    pub absorbs_mouse_events: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Layout {
    pub size: Xy<Size>,
    pub padding: Xy<Len>,
    pub position: Xy<Position>,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Rect {
    pub visible: bool,
    pub outline_only: bool,
    pub vertex_colors: VertexColors,
    // ... crazy stuff like texture and NinePatchRect
}
impl Rect {
    pub const DEFAULT: Self = Self {
        visible: true,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::FLGR_BLUE),
    };
}

// rename
// todo: add greyed text for textinput
#[derive(Debug, Copy, Clone, Hash)]
pub struct TextOptions {
    pub editable: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Image<'data> {
    pub data: &'data [u8],
}

pub(crate) const BASE_RADIUS: f32 = 20.0;

impl NodeParams {
    pub(crate) fn cosmetic_update_hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.rect.hash(&mut hasher);
        return hasher.finish();
    }

    pub(crate) fn partial_relayout_hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.layout.hash(&mut hasher);
        self.stack.hash(&mut hasher);
        self.text_params.hash(&mut hasher);
        return hasher.finish();
    }

    pub const fn const_default() -> Self {
        return DEFAULT;
    }

    pub const fn key(mut self, key: NodeKey) -> Self {
        self.key = key;
        return self;
    }

    // todo: in a future version of Rust that allows it, change these to take a generic Into<Size>
    pub const fn size_x(mut self, size: Size) -> Self {
        self.layout.size.x = size;
        return self;
    }
    pub const fn size_y(mut self, size: Size) -> Self {
        self.layout.size.y = size;
        return self;
    }
    pub const fn size_symm(mut self, size: Size) -> Self {
        self.layout.size = Xy::new_symm(size);
        return self;
    }

    pub const fn position_x(mut self, position: Position) -> Self {
        self.layout.position.x = position;
        return self;
    }
    pub const fn position_y(mut self, position: Position) -> Self {
        self.layout.position.y = position;
        return self;
    }
    pub const fn position_symm(mut self, position: Position) -> Self {
        self.layout.position = Xy::new_symm(position);
        return self;
    }

    pub const fn visible(mut self) -> Self {
        self.rect.visible = true;
        return self;
    }
    pub const fn invisible(mut self) -> Self {
        self.rect.visible = false;
        self.rect.outline_only = false;
        self.rect.vertex_colors = VertexColors::flat(Color::FLGR_DEBUG_RED);
        return self;
    }

    pub const fn filled(mut self, filled: bool) -> Self {
        self.rect.outline_only = filled;
        return self;
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.rect.vertex_colors = VertexColors::flat(color);
        return self;
    }

    pub const fn vertex_colors(mut self, colors: VertexColors) -> Self {
        self.rect.vertex_colors = colors;
        return self;
    }

    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: Len) -> Self {
        self.stack = Some(Stack {
            arrange,
            axis,
            spacing,
        });
        return self;
    }

    pub const fn stack_arrange(mut self, arrange: Arrange) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.arrange(arrange));
        return self;
    }

    pub const fn stack_spacing(mut self, spacing: Len) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.spacing(spacing));
        return self;
    }

    // todo: if we don't mind sacrificing symmetry, it could make sense to just remove this one.
    pub const fn stack_axis(mut self, axis: Axis) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.axis(axis));
        return self;
    }

    pub const fn padding(mut self, padding: Len) -> Self {
        self.layout.padding = Xy::new_symm(padding);
        return self;
    }

    pub const fn padding_x(mut self, padding: Len) -> Self {
        self.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: Len) -> Self {
        self.layout.padding.y = padding;
        return self;
    }

    pub const fn absorbs_clicks(mut self, absorbs_clicks: bool) -> Self {
        self.interact.absorbs_mouse_events = absorbs_clicks;
        return self;
    }

    pub fn is_fit_content(&self) -> bool {
        let Xy { x, y } = self.layout.size;
        return x == Size::FitContent || y == Size::FitContent
    }   
}