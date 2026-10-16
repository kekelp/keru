const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const lib_dir = b.option([]const u8, "keru-lib-dir", "Directory that contains libkeru_bindings, relative to the build root") orelse "../../../target/debug";

    const keru = b.addModule("keru", .{
        .root_source_file = b.path("src/keru.zig"),
        .target = target,
        .link_libc = true,
    });
    keru.addLibraryPath(b.path(lib_dir));
    keru.addRPathSpecial(b.fmt("$ORIGIN/../../{s}", .{lib_dir}));
    keru.linkSystemLibrary("keru_bindings", .{});

    const exe = b.addExecutable(.{
        .name = "counter",
        .root_module = b.createModule(.{
            .root_source_file = b.path("examples/counter.zig"),
            .target = target,
            .optimize = optimize,
            .imports = &.{
                .{ .name = "keru", .module = keru },
            },
        }),
    });
    b.installArtifact(exe);

    const run_step = b.step("run", "Run the counter example");
    const run_cmd = b.addRunArtifact(exe);
    run_step.dependOn(&run_cmd.step);
    run_cmd.step.dependOn(b.getInstallStep());
    run_cmd.addPassthruArgs();

    const mod_tests = b.addTest(.{
        .root_module = keru,
    });
    const run_mod_tests = b.addRunArtifact(mod_tests);

    const test_step = b.step("test", "Run tests");
    test_step.dependOn(&run_mod_tests.step);
}
