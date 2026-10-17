const keru = @import("keru.zig");

const Color = keru.Color;
const Vec2 = keru.Vec2;
const GridFlow = keru.GridFlow;
const Sense = keru.Sense;
const CursorIcon = keru.CursorIcon;
const HorizontalOrigin = keru.HorizontalOrigin;
const VerticalOrigin = keru.VerticalOrigin;
const WindowOverflow = keru.WindowOverflow;
const Axis = keru.Axis;
const Slide = keru.Slide;
const Role = keru.Role;
const ChildrenCanHide = keru.ChildrenCanHide;
const Transform = keru.Transform;
const GridElement = keru.GridElement;
const TextAlignment = keru.TextAlignment;
const VerticalTextAlignment = keru.VerticalTextAlignment;
const TextStyleFlags = keru.TextStyleFlags;
const NumericValue = keru.NumericValue;
const AccessibilityActions = keru.AccessibilityActions;

pub fn Opt(comptime T: type) type {
    return extern struct {
        has: bool,
        value: T,

        pub const none: @This() = .{ .has = false, .value = undefined };

        pub fn some(v: T) @This() {
            return .{ .has = true, .value = v };
        }
    };
}

pub fn Xy(comptime T: type) type {
    return extern struct { x: T, y: T };
}

pub const Str = extern struct {
    ptr: ?[*]const u8,
    len: usize,
};

pub const Size = extern struct {
    tag: enum(u32) { pixels, frac, fill, fit_content, aspect_ratio },
    payload: extern union { pixels: f32, frac: f32, aspect_ratio: f32 },
};

pub const Pos = extern struct {
    tag: enum(u32) { center, start, end, pixels, frac },
    payload: extern union { pixels: f32, frac: f32 },
};

pub const Anchor = extern struct {
    tag: enum(u32) { start, center, end, frac },
    payload: extern union { frac: f32 },
};

pub const MainAxisCellSize = extern struct {
    tag: enum(u32) { count, width },
    payload: extern union { count: u32, width: f32 },
};

pub const ChildrenLayout = extern struct {
    tag: enum(u32) { free, stack, grid },
    payload: extern union {
        stack: keru.ChildrenLayout.Stack,
        grid: extern struct { columns: MainAxisCellSize, spacing_x: f32, spacing_y: f32, flow: GridFlow },
    },
};

pub const Shape = extern struct {
    tag: enum(u32) { no_shape, rectangle, circle, ring, arc, pie, segment, horizontal_line, vertical_line, triangle, square_grid, hex_grid, hexagon },
    payload: extern union {
        rectangle: keru.Shape.Rectangle,
        ring: keru.Shape.Ring,
        arc: keru.Shape.Arc,
        pie: keru.Shape.Pie,
        segment: extern struct { start: Vec2, end: Vec2, dash_length: Opt(f32) },
        triangle: keru.Shape.Triangle,
        square_grid: keru.Shape.SquareGrid,
        hex_grid: keru.Shape.HexGrid,
        hexagon: keru.Shape.Hexagon,
    },
};

pub const ColorFill = extern struct {
    tag: enum(u32) { color, linear_gradient, radial_gradient, shared_gradient },
    payload: extern union {
        color: Color,
        linear_gradient: keru.ColorFill.LinearGradient,
        radial_gradient: keru.ColorFill.RadialGradient,
        shared_gradient: u64,
    },
};

pub const Shadow = extern struct {
    blur: f32,
    offset: Vec2,
    color: Opt(Color),
};

pub const Stroke = extern struct {
    width: f32,
    color: ColorFill,
    dash_length: f32,
    dash_offset: f32,
};

pub const TextOptions = extern struct {
    editable: bool,
    single_line: bool,
    selectable: bool,
    edit_disabled: bool,
    auto_markdown: bool,
    use_pointer_comparison: bool,
};

pub const Interact = extern struct {
    click_animation: bool,
    absorbs_mouse_events: bool,
    senses: Sense,
    show_focus_indicator: bool,
    focusable: bool,
    trap_keyboard_focus: bool,
    hover_cursor_icon: Opt(CursorIcon),
};

pub const Layout = extern struct {
    size: Xy(Size),
    min_size: Xy(Opt(Size)),
    max_size: Xy(Opt(Size)),
    padding: Xy(f32),
    position: Xy(Pos),
    anchor: Xy(Anchor),
    children_origin_x: HorizontalOrigin,
    children_origin_y: VerticalOrigin,
    scrollable: Xy(bool),
    window_overflow: Xy(WindowOverflow),
};

pub const AxisOrigin = extern struct { axis: Axis, origin: Pos };

pub const EnterAnimation = extern struct {
    tag: enum(u32) { none, slide, grow, fade_in },
    payload: extern union { slide: Slide, grow: AxisOrigin },
};

pub const ExitAnimation = extern struct {
    tag: enum(u32) { none, slide, shrink, fade_out },
    payload: extern union { slide: Slide, shrink: AxisOrigin },
};

pub const Animation = extern struct {
    speed: f32,
    enter: EnterAnimation,
    exit: ExitAnimation,
    state_transition: extern struct { animate_layout: bool, animate_properties: bool },
};

pub const Accessibility = extern struct {
    role: Role,
    selected: bool,
    numeric_value: Opt(NumericValue),
    actions: AccessibilityActions,
};

pub const Node = extern struct {
    key: Opt(u64),
    text_options: TextOptions,
    children_layout: ChildrenLayout,
    shape: Shape,
    blur: Opt(f32),
    shadow: Opt(Shadow),
    second_shadow: Opt(Shadow),
    stroke: Opt(Stroke),
    color: ColorFill,
    alpha: f32,
    visible: bool,
    interact: Interact,
    layout: Layout,
    children_can_hide: ChildrenCanHide,
    clip_children: Xy(bool),
    animation: Animation,
    transform: Transform,
    z_index: f32,
    grid_element: GridElement,
    free_placement: bool,
    ignore_parent_scroll: bool,
    text_alignment: TextAlignment,
    vertical_text_alignment: VerticalTextAlignment,
    text: Str,
    text_size: Opt(f32),
    text_color: Opt(Color),
    text_style_flags: TextStyleFlags,
    placeholder_text: Str,
    accessibility: Accessibility,
    constant: bool,
};
