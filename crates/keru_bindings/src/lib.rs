use std::ffi::c_void;
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::run_example_loop;

pub type KeruUi = Ui;

pub type KeruKey = u64;

pub type KeruUpdateFn = Option<extern "C" fn(ui: *mut KeruUi, user_data: *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruStr {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruOpt<T> {
    pub has: bool,
    pub value: T,
}

impl<T: Copy> KeruOpt<T> {
    fn get(self) -> Option<T> {
        if self.has { Some(self.value) } else { None }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruXy<T> {
    pub x: T,
    pub y: T,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruVec2 {
    pub x: f32,
    pub y: f32,
}

impl KeruVec2 {
    fn tuple(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruSize {
    Pixels(f32),
    Frac(f32),
    Fill,
    FitContent,
    AspectRatio(f32),
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruPos {
    Center,
    Start,
    End,
    Pixels(f32),
    Frac(f32),
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruAnchor {
    Start,
    Center,
    End,
    Frac(f32),
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruAxis {
    X,
    Y,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruArrange {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruDirection {
    LeftToRight,
    RightToLeft,
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruMainAxisCellSize {
    Count(u32),
    Width(f32),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruGridFlow {
    pub main_axis: KeruAxis,
    pub x_fill_direction: KeruDirection,
    pub y_fill_direction: KeruDirection,
    pub backfill: bool,
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruChildrenLayout {
    Free,
    Stack { arrange: KeruArrange, axis: KeruAxis, spacing: f32 },
    Grid { columns: KeruMainAxisCellSize, spacing_x: f32, spacing_y: f32, flow: KeruGridFlow },
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruShape {
    NoShape,
    Rectangle { rounded_corners: u32, corner_radius: f32 },
    Circle,
    Ring { width: f32 },
    Arc { start_angle: f32, end_angle: f32, width: f32 },
    Pie { start_angle: f32, end_angle: f32 },
    Segment { start: KeruVec2, end: KeruVec2, dash_length: KeruOpt<f32> },
    HorizontalLine,
    VerticalLine,
    Triangle { rotation: f32, width: f32 },
    SquareGrid { lattice_size: KeruVec2, offset: KeruVec2, line_thickness: f32 },
    HexGrid { lattice_size: f32, offset: KeruVec2, line_thickness: f32 },
    Hexagon { size: f32, rotation: f32 },
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruColorFill {
    Color(Color),
    LinearGradient { color_start: Color, color_end: Color, angle_deg: f32 },
    RadialGradient { color_inner: Color, color_outer: Color },
    SharedGradient(KeruKey),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruShadow {
    pub blur: f32,
    pub offset: KeruVec2,
    pub color: KeruOpt<Color>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruStroke {
    pub width: f32,
    pub color: KeruColorFill,
    pub dash_length: f32,
    pub dash_offset: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruTextOptions {
    pub editable: bool,
    pub single_line: bool,
    pub selectable: bool,
    pub edit_disabled: bool,
    pub auto_markdown: bool,
    pub use_pointer_comparison: bool,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruCursorIcon {
    Default,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
    AllScroll,
    ZoomIn,
    ZoomOut,
    DndAsk,
    AllResize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruInteract {
    pub click_animation: bool,
    pub absorbs_mouse_events: bool,
    pub senses: u16,
    pub show_focus_indicator: bool,
    pub focusable: bool,
    pub trap_keyboard_focus: bool,
    pub hover_cursor_icon: KeruOpt<KeruCursorIcon>,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruHorizontalOrigin {
    Left,
    Right,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruVerticalOrigin {
    Top,
    Bottom,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruWindowOverflow {
    Ignore,
    Clamp,
    Flip,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruLayout {
    pub size: KeruXy<KeruSize>,
    pub min_size: KeruXy<KeruOpt<KeruSize>>,
    pub max_size: KeruXy<KeruOpt<KeruSize>>,
    pub padding: KeruXy<f32>,
    pub position: KeruXy<KeruPos>,
    pub anchor: KeruXy<KeruAnchor>,
    pub children_origin_x: KeruHorizontalOrigin,
    pub children_origin_y: KeruVerticalOrigin,
    pub scrollable: KeruXy<bool>,
    pub window_overflow: KeruXy<KeruWindowOverflow>,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruChildrenCanHide {
    Yes,
    No,
    Inherit,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruSlideEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruSlideDirection {
    In,
    Out,
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruEnterAnimation {
    None,
    Slide { edge: KeruSlideEdge, direction: KeruSlideDirection },
    Grow { axis: KeruAxis, origin: KeruPos },
    FadeIn,
}

#[repr(C, u32)]
#[derive(Clone, Copy)]
pub enum KeruExitAnimation {
    None,
    Slide { edge: KeruSlideEdge, direction: KeruSlideDirection },
    Shrink { axis: KeruAxis, origin: KeruPos },
    FadeOut,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruStateTransition {
    pub animate_layout: bool,
    pub animate_properties: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruAnimation {
    pub speed: f32,
    pub enter: KeruEnterAnimation,
    pub exit: KeruExitAnimation,
    pub state_transition: KeruStateTransition,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruTransform {
    pub offset: KeruVec2,
    pub scale: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruGridElement {
    pub row_span: u16,
    pub column_span: u16,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruTextAlignment {
    Start,
    End,
    Left,
    Center,
    Right,
    Justify,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruVerticalTextAlignment {
    Center,
    Top,
    Bottom,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruNumericValue {
    pub value: f64,
    pub min: f64,
    pub max: f64,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum KeruRole {
    GenericContainer,
    Button,
    Label,
    TextInput,
    ScrollView,
    Grid,
    Image,
    Unknown,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruAccessibility {
    pub role: KeruRole,
    pub selected: bool,
    pub numeric_value: KeruOpt<KeruNumericValue>,
    pub actions: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruNode {
    pub key: KeruOpt<KeruKey>,
    pub text_options: KeruTextOptions,
    pub children_layout: KeruChildrenLayout,
    pub shape: KeruShape,
    pub blur: KeruOpt<f32>,
    pub shadow: KeruOpt<KeruShadow>,
    pub second_shadow: KeruOpt<KeruShadow>,
    pub stroke: KeruOpt<KeruStroke>,
    pub color: KeruColorFill,
    pub alpha: f32,
    pub visible: bool,
    pub interact: KeruInteract,
    pub layout: KeruLayout,
    pub children_can_hide: KeruChildrenCanHide,
    pub clip_children: KeruXy<bool>,
    pub animation: KeruAnimation,
    pub transform: KeruTransform,
    pub z_index: f32,
    pub grid_element: KeruGridElement,
    pub free_placement: bool,
    pub ignore_parent_scroll: bool,
    pub text_alignment: KeruTextAlignment,
    pub vertical_text_alignment: KeruVerticalTextAlignment,
    pub text: KeruStr,
    pub text_size: KeruOpt<f32>,
    pub text_color: KeruOpt<Color>,
    pub text_style_flags: u8,
    pub placeholder_text: KeruStr,
    pub accessibility: KeruAccessibility,
    pub constant: bool,
}

fn to_node_key(key: KeruKey) -> NodeKey {
    NodeKey::new(Id(key), "Bindings key")
}

impl KeruSize {
    fn to_keru(self) -> Size {
        match self {
            KeruSize::Pixels(v) => Size::Pixels(v),
            KeruSize::Frac(v) => Size::Frac(v),
            KeruSize::Fill => Size::Fill,
            KeruSize::FitContent => Size::FitContent,
            KeruSize::AspectRatio(v) => Size::AspectRatio(v),
        }
    }
}

impl KeruPos {
    fn to_keru(self) -> Pos {
        match self {
            KeruPos::Center => Pos::Center,
            KeruPos::Start => Pos::Start,
            KeruPos::End => Pos::End,
            KeruPos::Pixels(v) => Pos::Pixels(v),
            KeruPos::Frac(v) => Pos::Frac(v),
        }
    }
}

impl KeruAnchor {
    fn to_keru(self) -> Anchor {
        match self {
            KeruAnchor::Start => Anchor::Start,
            KeruAnchor::Center => Anchor::Center,
            KeruAnchor::End => Anchor::End,
            KeruAnchor::Frac(v) => Anchor::Frac(v),
        }
    }
}

impl KeruAxis {
    fn to_keru(self) -> Axis {
        match self {
            KeruAxis::X => Axis::X,
            KeruAxis::Y => Axis::Y,
        }
    }
}

impl KeruArrange {
    fn to_keru(self) -> Arrange {
        match self {
            KeruArrange::Start => Arrange::Start,
            KeruArrange::End => Arrange::End,
            KeruArrange::Center => Arrange::Center,
            KeruArrange::SpaceBetween => Arrange::SpaceBetween,
            KeruArrange::SpaceAround => Arrange::SpaceAround,
            KeruArrange::SpaceEvenly => Arrange::SpaceEvenly,
        }
    }
}

impl KeruDirection {
    fn to_keru(self) -> Direction {
        match self {
            KeruDirection::LeftToRight => Direction::LeftToRight,
            KeruDirection::RightToLeft => Direction::RightToLeft,
        }
    }
}

impl KeruMainAxisCellSize {
    fn to_keru(self) -> MainAxisCellSize {
        match self {
            KeruMainAxisCellSize::Count(n) => MainAxisCellSize::Count(n),
            KeruMainAxisCellSize::Width(w) => MainAxisCellSize::Width(w),
        }
    }
}

impl KeruGridFlow {
    fn to_keru(self) -> GridFlow {
        GridFlow {
            main_axis: self.main_axis.to_keru(),
            x_fill_direction: self.x_fill_direction.to_keru(),
            y_fill_direction: self.y_fill_direction.to_keru(),
            backfill: self.backfill,
        }
    }
}

impl KeruChildrenLayout {
    fn to_keru(self) -> ChildrenLayout {
        match self {
            KeruChildrenLayout::Free => ChildrenLayout::Free,
            KeruChildrenLayout::Stack { arrange, axis, spacing } => ChildrenLayout::Stack {
                arrange: arrange.to_keru(),
                axis: axis.to_keru(),
                spacing,
            },
            KeruChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => ChildrenLayout::Grid {
                columns: columns.to_keru(),
                spacing_x,
                spacing_y,
                flow: flow.to_keru(),
            },
        }
    }
}

impl KeruShape {
    fn to_keru(self) -> Shape {
        match self {
            KeruShape::NoShape => Shape::NoShape,
            KeruShape::Rectangle { rounded_corners, corner_radius } => Shape::Rectangle {
                rounded_corners: RoundedCorners::from_bits_truncate(rounded_corners),
                corner_radius,
            },
            KeruShape::Circle => Shape::Circle,
            KeruShape::Ring { width } => Shape::Ring { width },
            KeruShape::Arc { start_angle, end_angle, width } => Shape::Arc { start_angle, end_angle, width },
            KeruShape::Pie { start_angle, end_angle } => Shape::Pie { start_angle, end_angle },
            KeruShape::Segment { start, end, dash_length } => Shape::Segment {
                start: start.tuple(),
                end: end.tuple(),
                dash_length: dash_length.get(),
            },
            KeruShape::HorizontalLine => Shape::HorizontalLine,
            KeruShape::VerticalLine => Shape::VerticalLine,
            KeruShape::Triangle { rotation, width } => Shape::Triangle { rotation, width },
            KeruShape::SquareGrid { lattice_size, offset, line_thickness } => Shape::SquareGrid {
                lattice_size: lattice_size.tuple(),
                offset: offset.tuple(),
                line_thickness,
            },
            KeruShape::HexGrid { lattice_size, offset, line_thickness } => Shape::HexGrid {
                lattice_size,
                offset: offset.tuple(),
                line_thickness,
            },
            KeruShape::Hexagon { size, rotation } => Shape::Hexagon { size, rotation },
        }
    }
}

impl KeruColorFill {
    fn to_keru(self) -> ColorFill {
        match self {
            KeruColorFill::Color(c) => ColorFill::Color(c),
            KeruColorFill::LinearGradient { color_start, color_end, angle_deg } => {
                ColorFill::LinearGradient(LinearGradient::new(color_start, color_end, angle_deg))
            }
            KeruColorFill::RadialGradient { color_inner, color_outer } => ColorFill::RadialGradient { color_inner, color_outer },
            KeruColorFill::SharedGradient(key) => ColorFill::SharedGradient(to_node_key(key)),
        }
    }
}

impl KeruShadow {
    fn to_keru(self) -> Shadow {
        Shadow {
            blur: self.blur,
            offset: Xy::new(self.offset.x, self.offset.y),
            color: self.color.get(),
        }
    }
}

impl KeruStroke {
    fn to_keru(self) -> Stroke {
        Stroke {
            width: self.width,
            color: self.color.to_keru(),
            dash_length: self.dash_length,
            dash_offset: self.dash_offset,
        }
    }
}

impl KeruCursorIcon {
    fn to_keru(self) -> CursorIcon {
        match self {
            KeruCursorIcon::Default => CursorIcon::Default,
            KeruCursorIcon::ContextMenu => CursorIcon::ContextMenu,
            KeruCursorIcon::Help => CursorIcon::Help,
            KeruCursorIcon::Pointer => CursorIcon::Pointer,
            KeruCursorIcon::Progress => CursorIcon::Progress,
            KeruCursorIcon::Wait => CursorIcon::Wait,
            KeruCursorIcon::Cell => CursorIcon::Cell,
            KeruCursorIcon::Crosshair => CursorIcon::Crosshair,
            KeruCursorIcon::Text => CursorIcon::Text,
            KeruCursorIcon::VerticalText => CursorIcon::VerticalText,
            KeruCursorIcon::Alias => CursorIcon::Alias,
            KeruCursorIcon::Copy => CursorIcon::Copy,
            KeruCursorIcon::Move => CursorIcon::Move,
            KeruCursorIcon::NoDrop => CursorIcon::NoDrop,
            KeruCursorIcon::NotAllowed => CursorIcon::NotAllowed,
            KeruCursorIcon::Grab => CursorIcon::Grab,
            KeruCursorIcon::Grabbing => CursorIcon::Grabbing,
            KeruCursorIcon::EResize => CursorIcon::EResize,
            KeruCursorIcon::NResize => CursorIcon::NResize,
            KeruCursorIcon::NeResize => CursorIcon::NeResize,
            KeruCursorIcon::NwResize => CursorIcon::NwResize,
            KeruCursorIcon::SResize => CursorIcon::SResize,
            KeruCursorIcon::SeResize => CursorIcon::SeResize,
            KeruCursorIcon::SwResize => CursorIcon::SwResize,
            KeruCursorIcon::WResize => CursorIcon::WResize,
            KeruCursorIcon::EwResize => CursorIcon::EwResize,
            KeruCursorIcon::NsResize => CursorIcon::NsResize,
            KeruCursorIcon::NeswResize => CursorIcon::NeswResize,
            KeruCursorIcon::NwseResize => CursorIcon::NwseResize,
            KeruCursorIcon::ColResize => CursorIcon::ColResize,
            KeruCursorIcon::RowResize => CursorIcon::RowResize,
            KeruCursorIcon::AllScroll => CursorIcon::AllScroll,
            KeruCursorIcon::ZoomIn => CursorIcon::ZoomIn,
            KeruCursorIcon::ZoomOut => CursorIcon::ZoomOut,
            KeruCursorIcon::DndAsk => CursorIcon::DndAsk,
            KeruCursorIcon::AllResize => CursorIcon::AllResize,
        }
    }
}

impl KeruHorizontalOrigin {
    fn to_keru(self) -> HorizontalOrigin {
        match self {
            KeruHorizontalOrigin::Left => HorizontalOrigin::Left,
            KeruHorizontalOrigin::Right => HorizontalOrigin::Right,
        }
    }
}

impl KeruVerticalOrigin {
    fn to_keru(self) -> VerticalOrigin {
        match self {
            KeruVerticalOrigin::Top => VerticalOrigin::Top,
            KeruVerticalOrigin::Bottom => VerticalOrigin::Bottom,
        }
    }
}

impl KeruWindowOverflow {
    fn to_keru(self) -> WindowOverflow {
        match self {
            KeruWindowOverflow::Ignore => WindowOverflow::Ignore,
            KeruWindowOverflow::Clamp => WindowOverflow::Clamp,
            KeruWindowOverflow::Flip => WindowOverflow::Flip,
        }
    }
}

impl KeruChildrenCanHide {
    fn to_keru(self) -> ChildrenCanHide {
        match self {
            KeruChildrenCanHide::Yes => ChildrenCanHide::Yes,
            KeruChildrenCanHide::No => ChildrenCanHide::No,
            KeruChildrenCanHide::Inherit => ChildrenCanHide::Inherit,
        }
    }
}

impl KeruSlideEdge {
    fn to_keru(self) -> SlideEdge {
        match self {
            KeruSlideEdge::Top => SlideEdge::Top,
            KeruSlideEdge::Bottom => SlideEdge::Bottom,
            KeruSlideEdge::Left => SlideEdge::Left,
            KeruSlideEdge::Right => SlideEdge::Right,
        }
    }
}

impl KeruSlideDirection {
    fn to_keru(self) -> SlideDirection {
        match self {
            KeruSlideDirection::In => SlideDirection::In,
            KeruSlideDirection::Out => SlideDirection::Out,
        }
    }
}

impl KeruEnterAnimation {
    fn to_keru(self) -> EnterAnimation {
        match self {
            KeruEnterAnimation::None => EnterAnimation::None,
            KeruEnterAnimation::Slide { edge, direction } => EnterAnimation::Slide { edge: edge.to_keru(), direction: direction.to_keru() },
            KeruEnterAnimation::Grow { axis, origin } => EnterAnimation::Grow { axis: axis.to_keru(), origin: origin.to_keru() },
            KeruEnterAnimation::FadeIn => EnterAnimation::FadeIn,
        }
    }
}

impl KeruExitAnimation {
    fn to_keru(self) -> ExitAnimation {
        match self {
            KeruExitAnimation::None => ExitAnimation::None,
            KeruExitAnimation::Slide { edge, direction } => ExitAnimation::Slide { edge: edge.to_keru(), direction: direction.to_keru() },
            KeruExitAnimation::Shrink { axis, origin } => ExitAnimation::Shrink { axis: axis.to_keru(), origin: origin.to_keru() },
            KeruExitAnimation::FadeOut => ExitAnimation::FadeOut,
        }
    }
}

impl KeruTextAlignment {
    fn to_keru(self) -> TextAlignment {
        match self {
            KeruTextAlignment::Start => TextAlignment::Start,
            KeruTextAlignment::End => TextAlignment::End,
            KeruTextAlignment::Left => TextAlignment::Left,
            KeruTextAlignment::Center => TextAlignment::Center,
            KeruTextAlignment::Right => TextAlignment::Right,
            KeruTextAlignment::Justify => TextAlignment::Justify,
        }
    }
}

impl KeruVerticalTextAlignment {
    fn to_keru(self) -> VerticalTextAlignment {
        match self {
            KeruVerticalTextAlignment::Center => VerticalTextAlignment::Center,
            KeruVerticalTextAlignment::Top => VerticalTextAlignment::Top,
            KeruVerticalTextAlignment::Bottom => VerticalTextAlignment::Bottom,
        }
    }
}

impl KeruRole {
    fn to_keru(self) -> AccessKitRole {
        match self {
            KeruRole::GenericContainer => AccessKitRole::GenericContainer,
            KeruRole::Button => AccessKitRole::Button,
            KeruRole::Label => AccessKitRole::Label,
            KeruRole::TextInput => AccessKitRole::TextInput,
            KeruRole::ScrollView => AccessKitRole::ScrollView,
            KeruRole::Grid => AccessKitRole::Grid,
            KeruRole::Image => AccessKitRole::Image,
            KeruRole::Unknown => AccessKitRole::Unknown,
        }
    }
}

impl<T> KeruXy<T> {
    fn map<U>(self, f: impl Fn(T) -> U) -> Xy<U> {
        Xy::new(f(self.x), f(self.y))
    }
}

unsafe fn str_bytes<'a>(s: KeruStr) -> Option<&'a [u8]> {
    if s.ptr.is_null() {
        return None;
    }
    return Some(unsafe { std::slice::from_raw_parts(s.ptr, s.len) });
}

unsafe fn add(ui: &mut Ui, node: &KeruNode) -> UiParent {
    let text = unsafe { str_bytes(node.text) }.map(String::from_utf8_lossy);
    let placeholder_text = unsafe { str_bytes(node.placeholder_text) }.map(String::from_utf8_lossy);

    let to = &node.text_options;
    let i = &node.interact;
    let l = &node.layout;
    let a = &node.animation;
    let acc = &node.accessibility;

    let mut transform = DEFAULT.transform;
    transform.offset = Xy::new(node.transform.offset.x, node.transform.offset.y).into();
    transform.scale = node.transform.scale;

    let n = Node {
        key: node.key.get().map(to_node_key),
        text_options: TextOptions {
            editable: to.editable,
            single_line: to.single_line,
            selectable: to.selectable,
            edit_disabled: to.edit_disabled,
            auto_markdown: to.auto_markdown,
            use_pointer_comparison: to.use_pointer_comparison,
        },
        children_layout: node.children_layout.to_keru(),
        shape: node.shape.to_keru(),
        blur: node.blur.get(),
        shadow: node.shadow.get().map(KeruShadow::to_keru),
        second_shadow: node.second_shadow.get().map(KeruShadow::to_keru),
        stroke: node.stroke.get().map(KeruStroke::to_keru),
        color: node.color.to_keru(),
        alpha: node.alpha,
        visible: node.visible,
        interact: Interact {
            click_animation: i.click_animation,
            absorbs_mouse_events: i.absorbs_mouse_events,
            senses: Sense::from_bits_truncate(i.senses),
            show_focus_indicator: i.show_focus_indicator,
            focusable: i.focusable,
            trap_keyboard_focus: i.trap_keyboard_focus,
            hover_cursor_icon: i.hover_cursor_icon.get().map(KeruCursorIcon::to_keru),
        },
        layout: Layout {
            size: l.size.map(KeruSize::to_keru),
            min_size: l.min_size.map(|s| s.get().map(KeruSize::to_keru)),
            max_size: l.max_size.map(|s| s.get().map(KeruSize::to_keru)),
            padding: l.padding.map(|p| p),
            position: l.position.map(KeruPos::to_keru),
            anchor: l.anchor.map(KeruAnchor::to_keru),
            children_origin_x: l.children_origin_x.to_keru(),
            children_origin_y: l.children_origin_y.to_keru(),
            scrollable: l.scrollable.map(|s| s),
            window_overflow: l.window_overflow.map(KeruWindowOverflow::to_keru),
        },
        children_can_hide: node.children_can_hide.to_keru(),
        clip_children: node.clip_children.map(|c| c),
        animation: Animation {
            speed: a.speed,
            enter: a.enter.to_keru(),
            exit: a.exit.to_keru(),
            state_transition: StateTransition {
                animate_layout: a.state_transition.animate_layout,
                animate_properties: a.state_transition.animate_properties,
            },
        },
        transform,
        custom_render: false,
        z_index: node.z_index,
        grid_element: GridElement { row_span: node.grid_element.row_span, column_span: node.grid_element.column_span },
        free_placement: node.free_placement,
        ignore_parent_scroll: node.ignore_parent_scroll,
        text_alignment: node.text_alignment.to_keru(),
        vertical_text_alignment: node.vertical_text_alignment.to_keru(),
        text: text.as_deref().map(NodeText),
        text_size: node.text_size.get(),
        text_color: node.text_color.get(),
        text_properties: &[],
        text_style_flags: TextStyleFlags::from_bits_truncate(node.text_style_flags),
        image: None,
        image_options: ImageOptions::DEFAULT,
        placeholder_text: placeholder_text.as_deref().map(NodeText),
        accessibility: Accessibility {
            role: acc.role.to_keru(),
            selected: acc.selected,
            numeric_value: acc.numeric_value.get().map(|v| NumericValue { value: v.value, min: v.min, max: v.max }),
            actions: AccessibilityActions::from_bits_truncate(acc.actions),
        },
        constant: node.constant,
    };

    return ui.add(n);
}

struct CState {
    update: KeruUpdateFn,
    user_data: *mut c_void,
}

fn c_update(state: &mut CState, ui: &mut Ui) {
    if let Some(update) = state.update {
        update(ui, state.user_data);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn keru_run(update: KeruUpdateFn, user_data: *mut c_void) {
    run_example_loop(CState { update, user_data }, c_update);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_add(ui: *mut KeruUi, node: &KeruNode) -> KeruKey {
    let ui = unsafe { &mut *ui };
    let parent = unsafe { add(ui, node) };
    return parent.id(ui).0;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_add_begin(ui: *mut KeruUi, node: &KeruNode) {
    let ui = unsafe { &mut *ui };
    unsafe { add(ui, node) }.push_parent();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_end(ui: *mut KeruUi) {
    let ui = unsafe { &*ui };
    ui.pop_parent();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_is_clicked(ui: *mut KeruUi, key: KeruKey) -> bool {
    let ui = unsafe { &*ui };
    return ui.is_clicked(to_node_key(key));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_is_hovered(ui: *mut KeruUi, key: KeruKey) -> bool {
    let ui = unsafe { &*ui };
    return ui.is_hovered(to_node_key(key));
}
