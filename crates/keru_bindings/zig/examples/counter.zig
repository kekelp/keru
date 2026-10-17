const keru = @import("keru");

const State = struct {
    count: i32 = 0,
};

fn updateUi(state: *State, ui: *keru.Ui) !void {
    const increase_key = comptime keru.Key.fromLocation(@src());
    const decrease_key = comptime keru.Key.fromStr("decrease");

    const increase = keru.Node.button.update(.{ .key = increase_key, .text = "Increase", .color = .{ .color = .red } });
    const text = try ui.frame_arena.allocator().print("{d}", .{state.count}) ;
    const count = keru.Node.label.update(.{ .text = text });
    const decrease = keru.Node.button.update(.{ .key = decrease_key, .text = "Decrease" });

    ui.addParent(.v_stack);
    {
        ui.add(increase);
        ui.add(count);
        ui.add(decrease);
        // if (ui.addImmediate(decrease).isClicked()) state.count -= 1;
    }
    ui.endParent();

    if (ui.isClicked(increase_key)) state.count += 1;
    if (ui.isClicked(decrease_key)) state.count -= 1;
}

pub fn main() void {
    var state: State = .{};
    keru.run(&state, updateUi);
}
