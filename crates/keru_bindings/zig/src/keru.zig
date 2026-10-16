const std = @import("std");

pub const Key = enum(u64) {
    _,

    const fnv_offset: u64 = 14695981039346656037;
    const fnv_prime: u64 = 1099511628211;

    fn fnv(start: u64, bytes: []const u8) u64 {
        var hash = start;
        for (bytes) |b| {
            hash ^= b;
            hash *%= fnv_prime;
        }
        return hash;
    }

    fn fromHash(hash: u64) Key {
        return @enumFromInt(if (hash == 0) 1 else hash);
    }

    pub fn fromStr(s: []const u8) Key {
        return fromHash(fnv(fnv_offset, s));
    }

    pub fn fromLocation(comptime src: std.builtin.SourceLocation) Key {
        return comptime blk: {
            @setEvalBranchQuota(100_000);
            break :blk fromStr(src.file).sibling(src.line).sibling(src.column);
        };
    }

    pub fn sibling(self: Key, value: u64) Key {
        return fromHash(fnv(@intFromEnum(self), std.mem.asBytes(&std.mem.nativeToLittle(u64, value))));
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
};

pub const Base = enum(u32) {
    default = 0,
    v_stack = 1,
    h_stack = 2,
    v_scroll_stack = 3,
    h_scroll_stack = 4,
    button = 5,
    label = 6,
    text = 7,
    panel = 8,
    container = 9,
    spacer = 10,
};

pub const Node = struct {
    base: Base = .default,
    key: ?Key = null,
    text: ?[]const u8 = null,
    color: ?Color = null,

    fn toC(self: Node) CNode {
        return .{
            .base = @intFromEnum(self.base),
            .key = if (self.key) |k| @intFromEnum(k) else 0,
            .text = if (self.text) |t| .{ .ptr = t.ptr, .len = t.len } else .{ .ptr = null, .len = 0 },
            .has_color = self.color != null,
            .color = self.color orelse .black,
        };
    }
};

pub const Ui = struct {
    raw: *RawUi,
    frame_arena: std.heap.ArenaAllocator,

    pub fn add(ui: *Ui, node: Node) void {
        keru_add(ui.raw, node.toC());
    }

    pub fn addParent(ui: *Ui, node: Node) void {
        keru_add_begin(ui.raw, node.toC());
    }

    pub fn endParent(ui: *Ui) void {
        keru_end(ui.raw);
    }

    pub fn frameAllocator(ui: *Ui) std.mem.Allocator {
        return ui.frame_arena.allocator();
    }

    pub fn allocPrint(ui: *Ui, comptime format: []const u8, args: anytype) std.mem.Allocator.Error![]const u8 {
        return std.fmt.allocPrint(ui.frameAllocator(), format, args);
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

const RawUi = opaque {};

const CStr = extern struct {
    ptr: ?[*]const u8,
    len: usize,
};

const CNode = extern struct {
    base: u32,
    key: u64,
    text: CStr,
    has_color: bool,
    color: Color,
};

const UpdateFn = *const fn (ui: *RawUi, user_data: ?*anyopaque) callconv(.c) void;

extern fn keru_run(update: ?UpdateFn, user_data: ?*anyopaque) void;
extern fn keru_add(ui: *RawUi, node: CNode) void;
extern fn keru_add_begin(ui: *RawUi, node: CNode) void;
extern fn keru_end(ui: *RawUi) void;
extern fn keru_is_clicked(ui: *RawUi, key: u64) bool;
extern fn keru_is_hovered(ui: *RawUi, key: u64) bool;
