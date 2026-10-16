const keru = @import("keru");

const State = struct {
    count: i32 = 0,
};

fn updateUi(state: *State, ui: *keru.Ui) !void {
    const increase_key = comptime keru.Key.fromLocation(@src());
    const decrease_key = comptime keru.Key.fromStr("decrease");

    const v_stack: keru.Node = .{ .base = .v_stack };
    const increase: keru.Node = .{ .base = .button, .key = increase_key, .text = "Increase", .color = .red };
    const count: keru.Node = .{ .base = .label, .text = try ui.allocPrint("{d}", .{state.count}) };
    const decrease: keru.Node = .{ .base = .button, .key = decrease_key, .text = "Decrease" };

    ui.addParent(v_stack);
    {
        ui.add(increase);
        ui.add(count);
        ui.add(decrease);
    }
    ui.endParent();

    if (ui.isClicked(increase_key)) state.count += 1;
    if (ui.isClicked(decrease_key)) state.count -= 1;
}

pub fn main() void {
    var state: State = .{};
    keru.run(&state, updateUi);
}
