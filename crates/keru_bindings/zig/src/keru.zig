const std = @import("std");

pub const Key = enum(u64) {
    _,

    const Hasher = std.hash.Fnv1a_64;

    fn fromHash(hash: u64) Key {
        return @enumFromInt(if (hash == 0) 1 else hash);
    }

    pub fn fromStr(s: []const u8) Key {
        return fromHash(Hasher.hash(s));
    }

    pub fn fromLocation(comptime src: std.builtin.SourceLocation) Key {
        return comptime blk: {
            @setEvalBranchQuota(100_000);
            break :blk fromStr(src.file).sibling(src.line).sibling(src.column);
        };
    }

    pub fn sibling(self: Key, value: u64) Key {
        var hasher: Hasher = .{ .value = @intFromEnum(self) };
        hasher.update(std.mem.asBytes(&std.mem.nativeToLittle(u64, value)));
        return fromHash(hasher.final());
    }
};

pub const Color = extern struct {
    r: f32,
    g: f32,
    b: f32,
    a: f32 = 1.0,

    pub const red: Color = .{ .r = 1, .g = 0, .b = 0 };
    pub const green: Color = .{ .r = 0, .g = 1, .b = 0 };
    pub const blue: Color = .{ .r = 0, .g = 0, .b = 1 };
    pub const black: Color = .{ .r = 0, .g = 0, .b = 0 };
    pub const white: Color = .{ .r = 1, .g = 1, .b = 1 };
    pub const grey: Color = .{ .r = 0.2, .g = 0.2, .b = 0.2 };
    pub const transparent: Color = .{ .r = 1, .g = 1, .b = 1, .a = 0 };
    pub const keru_blue: Color = .{ .r = 0.31, .g = 0.31, .b = 1 };
    pub const keru_pink: Color = .{ .r = 0.65, .g = 0.31, .b = 0.65 };
};

pub const Vec2 = extern struct {
    x: f32,
    y: f32,
};

pub fn Xy(comptime T: type) type {
    return struct {
        x: T,
        y: T,

        pub fn symm(value: T) @This() {
            return .{ .x = value, .y = value };
        }
    };
}

pub const Size = union(enum) {
    pixels: f32,
    frac: f32,
    fill,
    fit_content,
    aspect_ratio: f32,

    fn toC(self: Size) c.Size {
        return switch (self) {
            .pixels => |v| .{ .tag = .pixels, .payload = .{ .pixels = v } },
            .frac => |v| .{ .tag = .frac, .payload = .{ .frac = v } },
            .fill => .{ .tag = .fill, .payload = undefined },
            .fit_content => .{ .tag = .fit_content, .payload = undefined },
            .aspect_ratio => |v| .{ .tag = .aspect_ratio, .payload = .{ .aspect_ratio = v } },
        };
    }
};

pub const Pos = union(enum) {
    center,
    start,
    end,
    pixels: f32,
    frac: f32,

    fn toC(self: Pos) c.Pos {
        return switch (self) {
            .center => .{ .tag = .center, .payload = undefined },
            .start => .{ .tag = .start, .payload = undefined },
            .end => .{ .tag = .end, .payload = undefined },
            .pixels => |v| .{ .tag = .pixels, .payload = .{ .pixels = v } },
            .frac => |v| .{ .tag = .frac, .payload = .{ .frac = v } },
        };
    }
};

pub const Anchor = union(enum) {
    start,
    center,
    end,
    frac: f32,

    fn toC(self: Anchor) c.Anchor {
        return switch (self) {
            .start => .{ .tag = .start, .payload = undefined },
            .center => .{ .tag = .center, .payload = undefined },
            .end => .{ .tag = .end, .payload = undefined },
            .frac => |v| .{ .tag = .frac, .payload = .{ .frac = v } },
        };
    }
};

pub const Axis = enum(u32) { x, y };

pub const Arrange = enum(u32) { start, end, center, space_between, space_around, space_evenly };

pub const Direction = enum(u32) { left_to_right, right_to_left };

pub const MainAxisCellSize = union(enum) {
    count: u32,
    width: f32,

    fn toC(self: MainAxisCellSize) c.MainAxisCellSize {
        return switch (self) {
            .count => |v| .{ .tag = .count, .payload = .{ .count = v } },
            .width => |v| .{ .tag = .width, .payload = .{ .width = v } },
        };
    }
};

pub const GridFlow = extern struct {
    main_axis: Axis = .x,
    x_fill_direction: Direction = .left_to_right,
    y_fill_direction: Direction = .left_to_right,
    backfill: bool = false,
};

pub const ChildrenLayout = union(enum) {
    free,
    stack: Stack,
    grid: Grid,

    pub const Stack = extern struct {
        arrange: Arrange,
        axis: Axis,
        spacing: f32,
    };

    pub const Grid = struct {
        columns: MainAxisCellSize,
        spacing_x: f32,
        spacing_y: f32,
        flow: GridFlow = .{},
    };

    fn toC(self: ChildrenLayout) c.ChildrenLayout {
        return switch (self) {
            .free => .{ .tag = .free, .payload = undefined },
            .stack => |v| .{ .tag = .stack, .payload = .{ .stack = v } },
            .grid => |v| .{ .tag = .grid, .payload = .{ .grid = .{
                .columns = v.columns.toC(),
                .spacing_x = v.spacing_x,
                .spacing_y = v.spacing_y,
                .flow = v.flow,
            } } },
        };
    }
};

pub const RoundedCorners = packed struct(u32) {
    top_left: bool = false,
    top_right: bool = false,
    bottom_left: bool = false,
    bottom_right: bool = false,
    _padding: u28 = 0,

    pub const all: RoundedCorners = .{ .top_left = true, .top_right = true, .bottom_left = true, .bottom_right = true };
    pub const none: RoundedCorners = .{};
    pub const top: RoundedCorners = .{ .top_left = true, .top_right = true };
    pub const bottom: RoundedCorners = .{ .bottom_left = true, .bottom_right = true };
    pub const left: RoundedCorners = .{ .top_left = true, .bottom_left = true };
    pub const right: RoundedCorners = .{ .top_right = true, .bottom_right = true };
};

pub const Shape = union(enum) {
    no_shape,
    rectangle: Rectangle,
    circle,
    ring: Ring,
    arc: Arc,
    pie: Pie,
    segment: Segment,
    horizontal_line,
    vertical_line,
    triangle: Triangle,
    square_grid: SquareGrid,
    hex_grid: HexGrid,
    hexagon: Hexagon,

    pub const Rectangle = extern struct { rounded_corners: RoundedCorners, corner_radius: f32 };
    pub const Ring = extern struct { width: f32 };
    pub const Arc = extern struct { start_angle: f32, end_angle: f32, width: f32 };
    pub const Pie = extern struct { start_angle: f32, end_angle: f32 };
    pub const Segment = struct { start: Vec2, end: Vec2, dash_length: ?f32 = null };
    pub const Triangle = extern struct { rotation: f32, width: f32 };
    pub const SquareGrid = extern struct { lattice_size: Vec2, offset: Vec2, line_thickness: f32 };
    pub const HexGrid = extern struct { lattice_size: f32, offset: Vec2, line_thickness: f32 };
    pub const Hexagon = extern struct { size: f32, rotation: f32 };

    fn toC(self: Shape) c.Shape {
        return switch (self) {
            .no_shape => .{ .tag = .no_shape, .payload = undefined },
            .rectangle => |v| .{ .tag = .rectangle, .payload = .{ .rectangle = v } },
            .circle => .{ .tag = .circle, .payload = undefined },
            .ring => |v| .{ .tag = .ring, .payload = .{ .ring = v } },
            .arc => |v| .{ .tag = .arc, .payload = .{ .arc = v } },
            .pie => |v| .{ .tag = .pie, .payload = .{ .pie = v } },
            .segment => |v| .{ .tag = .segment, .payload = .{ .segment = .{
                .start = v.start,
                .end = v.end,
                .dash_length = opt(f32, v.dash_length),
            } } },
            .horizontal_line => .{ .tag = .horizontal_line, .payload = undefined },
            .vertical_line => .{ .tag = .vertical_line, .payload = undefined },
            .triangle => |v| .{ .tag = .triangle, .payload = .{ .triangle = v } },
            .square_grid => |v| .{ .tag = .square_grid, .payload = .{ .square_grid = v } },
            .hex_grid => |v| .{ .tag = .hex_grid, .payload = .{ .hex_grid = v } },
            .hexagon => |v| .{ .tag = .hexagon, .payload = .{ .hexagon = v } },
        };
    }
};

pub const ColorFill = union(enum) {
    color: Color,
    linear_gradient: LinearGradient,
    radial_gradient: RadialGradient,
    shared_gradient: Key,

    pub const LinearGradient = extern struct { color_start: Color, color_end: Color, angle_deg: f32 };
    pub const RadialGradient = extern struct { color_inner: Color, color_outer: Color };

    fn toC(self: ColorFill) c.ColorFill {
        return switch (self) {
            .color => |v| .{ .tag = .color, .payload = .{ .color = v } },
            .linear_gradient => |v| .{ .tag = .linear_gradient, .payload = .{ .linear_gradient = v } },
            .radial_gradient => |v| .{ .tag = .radial_gradient, .payload = .{ .radial_gradient = v } },
            .shared_gradient => |v| .{ .tag = .shared_gradient, .payload = .{ .shared_gradient = @intFromEnum(v) } },
        };
    }
};

pub const Shadow = struct {
    blur: f32,
    offset: Vec2,
    color: ?Color = null,

    fn toC(self: Shadow) c.Shadow {
        return .{ .blur = self.blur, .offset = self.offset, .color = opt(Color, self.color) };
    }
};

pub const Stroke = struct {
    width: f32,
    color: ColorFill,
    dash_length: f32 = 0,
    dash_offset: f32 = 0,

    fn toC(self: Stroke) c.Stroke {
        return .{ .width = self.width, .color = self.color.toC(), .dash_length = self.dash_length, .dash_offset = self.dash_offset };
    }
};

pub const TextOptions = struct {
    editable: bool = false,
    single_line: bool = false,
    selectable: bool = true,
    edit_disabled: bool = false,
    auto_markdown: bool = false,
    use_pointer_comparison: bool = false,

    fn toC(self: TextOptions) c.TextOptions {
        return .{
            .editable = self.editable,
            .single_line = self.single_line,
            .selectable = self.selectable,
            .edit_disabled = self.edit_disabled,
            .auto_markdown = self.auto_markdown,
            .use_pointer_comparison = self.use_pointer_comparison,
        };
    }
};

pub const Sense = packed struct(u16) {
    click: bool = false,
    drag: bool = false,
    hover: bool = false,
    scroll: bool = false,
    hold: bool = false,
    click_release: bool = false,
    drag_drop_target: bool = false,
    time: bool = false,
    hover_enter_or_exit: bool = false,
    _padding: u7 = 0,
};

pub const CursorIcon = enum(u32) {
    default,
    context_menu,
    help,
    pointer,
    progress,
    wait,
    cell,
    crosshair,
    text,
    vertical_text,
    alias,
    copy,
    move,
    no_drop,
    not_allowed,
    grab,
    grabbing,
    e_resize,
    n_resize,
    ne_resize,
    nw_resize,
    s_resize,
    se_resize,
    sw_resize,
    w_resize,
    ew_resize,
    ns_resize,
    nesw_resize,
    nwse_resize,
    col_resize,
    row_resize,
    all_scroll,
    zoom_in,
    zoom_out,
    dnd_ask,
    all_resize,
};

pub const Interact = struct {
    click_animation: bool = false,
    absorbs_mouse_events: bool = false,
    senses: Sense = .{},
    show_focus_indicator: bool = true,
    focusable: bool = false,
    trap_keyboard_focus: bool = false,
    hover_cursor_icon: ?CursorIcon = null,

    fn toC(self: Interact) c.Interact {
        return .{
            .click_animation = self.click_animation,
            .absorbs_mouse_events = self.absorbs_mouse_events,
            .senses = self.senses,
            .show_focus_indicator = self.show_focus_indicator,
            .focusable = self.focusable,
            .trap_keyboard_focus = self.trap_keyboard_focus,
            .hover_cursor_icon = opt(CursorIcon, self.hover_cursor_icon),
        };
    }
};

pub const HorizontalOrigin = enum(u32) { left, right };

pub const VerticalOrigin = enum(u32) { top, bottom };

pub const WindowOverflow = enum(u32) { ignore, clamp, flip };

pub const Layout = struct {
    size: Xy(Size) = .symm(.{ .frac = 1 }),
    min_size: Xy(?Size) = .symm(null),
    max_size: Xy(?Size) = .symm(null),
    padding: Xy(f32) = .symm(0),
    position: Xy(Pos) = .symm(.center),
    anchor: Xy(Anchor) = .symm(.start),
    children_origin_x: HorizontalOrigin = .left,
    children_origin_y: VerticalOrigin = .top,
    scrollable: Xy(bool) = .symm(false),
    window_overflow: Xy(WindowOverflow) = .symm(.ignore),

    fn toC(self: Layout) c.Layout {
        return .{
            .size = .{ .x = self.size.x.toC(), .y = self.size.y.toC() },
            .min_size = .{ .x = optMap(c.Size, self.min_size.x), .y = optMap(c.Size, self.min_size.y) },
            .max_size = .{ .x = optMap(c.Size, self.max_size.x), .y = optMap(c.Size, self.max_size.y) },
            .padding = xy(f32, self.padding),
            .position = .{ .x = self.position.x.toC(), .y = self.position.y.toC() },
            .anchor = .{ .x = self.anchor.x.toC(), .y = self.anchor.y.toC() },
            .children_origin_x = self.children_origin_x,
            .children_origin_y = self.children_origin_y,
            .scrollable = xy(bool, self.scrollable),
            .window_overflow = xy(WindowOverflow, self.window_overflow),
        };
    }
};

pub const ChildrenCanHide = enum(u32) { yes, no, inherit };

pub const SlideEdge = enum(u32) { top, bottom, left, right };

pub const SlideDirection = enum(u32) { in, out };

pub const Slide = extern struct { edge: SlideEdge, direction: SlideDirection };

pub const AxisOrigin = struct {
    axis: Axis,
    origin: Pos,

    fn toC(self: AxisOrigin) c.AxisOrigin {
        return .{ .axis = self.axis, .origin = self.origin.toC() };
    }
};

pub const EnterAnimation = union(enum) {
    none,
    slide: Slide,
    grow: AxisOrigin,
    fade_in,

    fn toC(self: EnterAnimation) c.EnterAnimation {
        return switch (self) {
            .none => .{ .tag = .none, .payload = undefined },
            .slide => |v| .{ .tag = .slide, .payload = .{ .slide = v } },
            .grow => |v| .{ .tag = .grow, .payload = .{ .grow = v.toC() } },
            .fade_in => .{ .tag = .fade_in, .payload = undefined },
        };
    }
};

pub const ExitAnimation = union(enum) {
    none,
    slide: Slide,
    shrink: AxisOrigin,
    fade_out,

    fn toC(self: ExitAnimation) c.ExitAnimation {
        return switch (self) {
            .none => .{ .tag = .none, .payload = undefined },
            .slide => |v| .{ .tag = .slide, .payload = .{ .slide = v } },
            .shrink => |v| .{ .tag = .shrink, .payload = .{ .shrink = v.toC() } },
            .fade_out => .{ .tag = .fade_out, .payload = undefined },
        };
    }
};

pub const StateTransition = struct {
    animate_layout: bool = false,
    animate_properties: bool = false,
};

pub const Animation = struct {
    speed: f32 = 1,
    enter: EnterAnimation = .none,
    exit: ExitAnimation = .none,
    state_transition: StateTransition = .{},

    fn toC(self: Animation) c.Animation {
        return .{
            .speed = self.speed,
            .enter = self.enter.toC(),
            .exit = self.exit.toC(),
            .state_transition = .{
                .animate_layout = self.state_transition.animate_layout,
                .animate_properties = self.state_transition.animate_properties,
            },
        };
    }
};

pub const Transform = extern struct {
    offset: Vec2 = .{ .x = 0, .y = 0 },
    scale: f32 = 1,
};

pub const GridElement = extern struct {
    row_span: u16 = 1,
    column_span: u16 = 1,
};

pub const TextAlignment = enum(u32) { start, end, left, center, right, justify };

pub const VerticalTextAlignment = enum(u32) { center, top, bottom };

pub const TextStyleFlags = packed struct(u8) {
    bold: bool = false,
    italic: bool = false,
    monospace: bool = false,
    _padding: u5 = 0,
};

pub const NumericValue = extern struct {
    value: f64,
    min: f64,
    max: f64,
};

pub const AccessibilityActions = packed struct(u32) {
    click: bool = false,
    focus: bool = false,
    blur: bool = false,
    collapse: bool = false,
    expand: bool = false,
    custom_action: bool = false,
    decrement: bool = false,
    increment: bool = false,
    hide_tooltip: bool = false,
    show_tooltip: bool = false,
    replace_selected_text: bool = false,
    scroll_down: bool = false,
    scroll_left: bool = false,
    scroll_right: bool = false,
    scroll_up: bool = false,
    scroll_into_view: bool = false,
    scroll_to_point: bool = false,
    set_scroll_offset: bool = false,
    set_text_selection: bool = false,
    set_sequential_focus_navigation_starting_point: bool = false,
    set_value: bool = false,
    show_context_menu: bool = false,
    _padding: u10 = 0,
};

pub const Role = enum(u32) { generic_container, button, label, text_input, scroll_view, grid, image, unknown };

pub const Accessibility = struct {
    role: Role = .generic_container,
    selected: bool = false,
    numeric_value: ?NumericValue = null,
    actions: AccessibilityActions = .{},

    fn toC(self: Accessibility) c.Accessibility {
        return .{
            .role = self.role,
            .selected = self.selected,
            .numeric_value = opt(NumericValue, self.numeric_value),
            .actions = self.actions,
        };
    }
};

pub const Node = struct {
    key: ?Key = null,
    text_options: TextOptions = .{},
    children_layout: ChildrenLayout = .free,
    shape: Shape = .{ .rectangle = .{ .rounded_corners = .all, .corner_radius = default_corner_radius } },
    blur: ?f32 = null,
    shadow: ?Shadow = null,
    second_shadow: ?Shadow = null,
    stroke: ?Stroke = null,
    color: ColorFill = .{ .color = .keru_blue },
    alpha: f32 = 1,
    visible: bool = true,
    interact: Interact = .{},
    layout: Layout = .{},
    children_can_hide: ChildrenCanHide = .inherit,
    clip_children: Xy(bool) = .symm(false),
    animation: Animation = .{},
    transform: Transform = .{},
    z_index: f32 = 0,
    grid_element: GridElement = .{},
    free_placement: bool = false,
    ignore_parent_scroll: bool = false,
    text_alignment: TextAlignment = .center,
    vertical_text_alignment: VerticalTextAlignment = .center,
    text: ?[]const u8 = null,
    text_size: ?f32 = null,
    text_color: ?Color = null,
    text_style_flags: TextStyleFlags = .{},
    placeholder_text: ?[]const u8 = null,
    accessibility: Accessibility = .{},
    constant: bool = false,

    pub const default: Node = .{};

    pub const v_stack: Node = .{
        .children_layout = .{ .stack = .{ .arrange = .center, .axis = .y, .spacing = 8 } },
        .color = .{ .color = .transparent },
        .layout = .{ .size = .symm(.fit_content) },
    };

    pub const h_stack: Node = .{
        .children_layout = .{ .stack = .{ .arrange = .center, .axis = .x, .spacing = 8 } },
        .color = .{ .color = .transparent },
        .layout = .{ .size = .symm(.fit_content) },
    };

    pub const v_scroll_stack: Node = .{
        .children_layout = .{ .stack = .{ .arrange = .start, .axis = .y, .spacing = 10 } },
        .color = .{ .color = .transparent },
        .layout = .{ .size = .{ .x = .fit_content, .y = .fill }, .scrollable = .{ .x = false, .y = true } },
        .clip_children = .{ .x = false, .y = true },
        .accessibility = .{ .role = .scroll_view },
    };

    pub const button: Node = .{
        .text_options = .{ .single_line = true, .selectable = false },
        .color = .{ .color = .keru_pink },
        .interact = .{
            .senses = .{ .click = true, .click_release = true },
            .absorbs_mouse_events = true,
            .click_animation = true,
            .focusable = true,
            .hover_cursor_icon = .pointer,
        },
        .layout = .{ .size = .symm(.fit_content), .padding = .symm(10) },
        .accessibility = .{ .role = .button },
    };

    pub const label: Node = .{
        .text_options = .{ .single_line = true },
        .interact = .{ .absorbs_mouse_events = true },
        .layout = .{ .size = .symm(.fit_content), .padding = .symm(10) },
        .accessibility = .{ .role = .label },
    };

    pub const text_node: Node = .{
        .text_options = .{ .single_line = true },
        .color = .{ .color = .transparent },
        .interact = .{ .hover_cursor_icon = .text },
        .layout = .{ .size = .symm(.fit_content), .padding = .symm(2) },
        .accessibility = .{ .role = .label },
    };

    pub const panel: Node = .{
        .color = .{ .color = .keru_pink },
        .interact = .{ .absorbs_mouse_events = true },
        .layout = .{ .size = .symm(.fit_content), .padding = .symm(10) },
    };

    pub const container: Node = .{
        .visible = false,
        .color = .{ .color = .transparent },
        .layout = .{ .size = .symm(.fit_content), .padding = .symm(10) },
    };

    pub const spacer: Node = .{
        .color = .{ .color = .transparent },
        .layout = .{ .size = .symm(.fill) },
        .accessibility = .{ .role = .unknown },
    };

    pub const grid: Node = .{
        .children_layout = .{ .grid = .{ .columns = .{ .count = 3 }, .spacing_x = 8, .spacing_y = 8 } },
        .color = .{ .color = .transparent },
        .layout = .{ .size = .symm(.fit_content) },
        .accessibility = .{ .role = .grid },
    };

    pub fn update(self: Node, updates: NodeUpdates) Node {
        var node = self;
        merge(&node, updates);
        return node;
    }

    fn toC(self: Node) c.Node {
        return .{
            .key = if (self.key) |k| .some(@intFromEnum(k)) else .none,
            .text_options = self.text_options.toC(),
            .children_layout = self.children_layout.toC(),
            .shape = self.shape.toC(),
            .blur = opt(f32, self.blur),
            .shadow = optMap(c.Shadow, self.shadow),
            .second_shadow = optMap(c.Shadow, self.second_shadow),
            .stroke = optMap(c.Stroke, self.stroke),
            .color = self.color.toC(),
            .alpha = self.alpha,
            .visible = self.visible,
            .interact = self.interact.toC(),
            .layout = self.layout.toC(),
            .children_can_hide = self.children_can_hide,
            .clip_children = xy(bool, self.clip_children),
            .animation = self.animation.toC(),
            .transform = self.transform,
            .z_index = self.z_index,
            .grid_element = self.grid_element,
            .free_placement = self.free_placement,
            .ignore_parent_scroll = self.ignore_parent_scroll,
            .text_alignment = self.text_alignment,
            .vertical_text_alignment = self.vertical_text_alignment,
            .text = str(self.text),
            .text_size = opt(f32, self.text_size),
            .text_color = opt(Color, self.text_color),
            .text_style_flags = self.text_style_flags,
            .placeholder_text = str(self.placeholder_text),
            .accessibility = self.accessibility.toC(),
            .constant = self.constant,
        };
    }
};

pub const NodeUpdates = Updates(Node);

pub fn Updates(comptime T: type) type {
    const info = @typeInfo(T).@"struct";
    var types: [info.field_names.len]type = undefined;
    var attrs: [info.field_names.len]std.builtin.Type.Struct.FieldAttributes = undefined;
    for (info.field_types, &types, &attrs) |F, *U, *attr| {
        if (@typeInfo(F) == .@"struct") {
            const default: Updates(F) = .{};
            U.* = Updates(F);
            attr.* = .{ .default_value_ptr = &default };
        } else {
            const default: ?F = null;
            U.* = ?F;
            attr.* = .{ .default_value_ptr = &default };
        }
    }
    return @Struct(.auto, null, info.field_names, &types, &attrs);
}

fn merge(target: anytype, updates: anytype) void {
    inline for (comptime std.meta.fieldNames(@TypeOf(updates))) |name| {
        const value = @field(updates, name);
        if (@typeInfo(@TypeOf(value)) == .optional) {
            if (value) |v| @field(target.*, name) = v;
        } else {
            merge(&@field(target.*, name), value);
        }
    }
}

pub const default_corner_radius: f32 = 9;

pub const UiNode = struct {
    ui: *Ui,
    key: Key,

    pub fn isClicked(self: UiNode) bool {
        return self.ui.isClicked(self.key);
    }

    pub fn isHovered(self: UiNode) bool {
        return self.ui.isHovered(self.key);
    }
};

pub const Ui = struct {
    raw: *RawUi,
    frame_arena: std.heap.ArenaAllocator,

    pub fn add(ui: *Ui, node: Node) void {
        _ = ui.addImmediate(node);
    }

    pub fn addImmediate(ui: *Ui, node: Node) UiNode {
        const c_node = node.toC();
        return .{ .ui = ui, .key = @enumFromInt(keru_add(ui.raw, &c_node)) };
    }

    pub fn addParent(ui: *Ui, node: Node) void {
        const c_node = node.toC();
        keru_add_begin(ui.raw, &c_node);
    }

    pub fn endParent(ui: *Ui) void {
        keru_end(ui.raw);
    }

    pub fn isClicked(ui: *Ui, key: Key) bool {
        return keru_is_clicked(ui.raw, @intFromEnum(key));
    }

    pub fn isHovered(ui: *Ui, key: Key) bool {
        return keru_is_hovered(ui.raw, @intFromEnum(key));
    }
};

pub fn run(state: anytype, comptime update: anytype) void {
    const Context = struct {
        state: @TypeOf(state),
        ui: Ui,

        fn cb(raw: *RawUi, user_data: ?*anyopaque) callconv(.c) void {
            const ctx: *@This() = @ptrCast(@alignCast(user_data.?));
            ctx.ui.raw = raw;
            _ = ctx.ui.frame_arena.reset(.retain_capacity);

            const result = update(ctx.state, &ctx.ui);
            if (@typeInfo(@TypeOf(result)) == .error_union) {
                result catch |err| std.debug.panic("keru update function returned an error: {s}", .{@errorName(err)});
            }
        }
    };

    var ctx: Context = .{
        .state = state,
        .ui = .{ .raw = undefined, .frame_arena = .init(std.heap.page_allocator) },
    };
    defer ctx.ui.frame_arena.deinit();

    keru_run(&Context.cb, &ctx);
}

fn opt(comptime T: type, value: ?T) c.Opt(T) {
    return if (value) |v| .some(v) else .none;
}

fn optMap(comptime C: type, value: anytype) c.Opt(C) {
    return if (value) |v| .some(v.toC()) else .none;
}

fn xy(comptime T: type, value: Xy(T)) c.Xy(T) {
    return .{ .x = value.x, .y = value.y };
}

fn str(value: ?[]const u8) c.Str {
    return if (value) |s| .{ .ptr = s.ptr, .len = s.len } else .{ .ptr = null, .len = 0 };
}

const c = @import("c.zig");

const RawUi = opaque {};

const UpdateFn = *const fn (ui: *RawUi, user_data: ?*anyopaque) callconv(.c) void;

extern fn keru_run(update: ?UpdateFn, user_data: ?*anyopaque) void;
extern fn keru_add(ui: *RawUi, node: *const c.Node) u64;
extern fn keru_add_begin(ui: *RawUi, node: *const c.Node) void;
extern fn keru_end(ui: *RawUi) void;
extern fn keru_is_clicked(ui: *RawUi, key: u64) bool;
extern fn keru_is_hovered(ui: *RawUi, key: u64) bool;
