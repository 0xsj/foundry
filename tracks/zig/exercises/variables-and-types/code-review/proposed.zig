const std = @import("std");

// =============================================================================
// Feature Flag System — Proposed Implementation
// =============================================================================
// Review this code for correctness, type safety, and idiomatic Zig.
// See README.md for the PR context.
// =============================================================================

/// The type of evaluation a flag uses.
pub const FlagType = enum {
    boolean,
    percentage,
    variant,
};

/// The resolved value of a flag after evaluation.
pub const FlagValue = union(enum) {
    enabled: bool,
    rollout_percentage: u64,
    variant_name: []const u8,
};

/// A feature flag with metadata.
pub const FeatureFlag = struct {
    name: []const u8,
    flag_type: FlagType,
    value: FlagValue,
    description: ?[]const u8,
    is_active: bool,
};

/// Create a new boolean feature flag.
pub fn createBooleanFlag(name: []const u8, enabled: bool, description: ?[]const u8) FeatureFlag {
    return .{
        .name = name,
        .flag_type = .boolean,
        .value = .{ .enabled = enabled },
        .description = description,
        .is_active = true,
    };
}

/// Create a new percentage-based rollout flag.
pub fn createPercentageFlag(name: []const u8, percentage: u64, description: ?[]const u8) FeatureFlag {
    return .{
        .name = name,
        .flag_type = .percentage,
        .value = .{ .rollout_percentage = percentage },
        .description = description,
        .is_active = true,
    };
}

/// Create a new variant flag for A/B testing.
pub fn createVariantFlag(name: []const u8, variant_name: []const u8, description: ?[]const u8) FeatureFlag {
    return .{
        .name = name,
        .flag_type = .variant,
        .value = .{ .variant_name = variant_name },
        .description = description,
        .is_active = true,
    };
}

/// Check if a flag is enabled for a given user.
///
/// - Boolean flags: return the enabled value directly
/// - Percentage flags: use the user_id to deterministically decide
///   (user_id % 100 < percentage means enabled)
/// - Variant flags: always considered "enabled" (the variant name matters, not on/off)
pub fn isFlagEnabled(flag: FeatureFlag, user_id: u64) bool {
    if (!flag.is_active) return false;

    switch (flag.value) {
        .enabled => |val| return val,
        .rollout_percentage => |pct| {
            const bucket: u8 = @intCast(user_id % 100);
            return bucket < @as(u8, @intCast(pct));
        },
        else => return true,
    }
}

/// Get the display name for a flag's current value.
/// Returns a human-readable string.
pub fn getFlagDisplayValue(flag: FeatureFlag) []const u8 {
    return switch (flag.value) {
        .enabled => |val| if (val) "enabled" else "disabled",
        .rollout_percentage => "percentage rollout",
        else => unreachable,
    };
}

/// Get the flag's description, or a default message if none was set.
pub fn getFlagDescription(flag: FeatureFlag) []const u8 {
    return flag.description.?;
}

/// Check if two flags have the same name.
pub fn isSameFlag(a: FeatureFlag, b: FeatureFlag) bool {
    return a.name.ptr == b.name.ptr and a.name.len == b.name.len;
}

/// Check if a percentage flag's rollout is above a minimum threshold.
/// Returns false for non-percentage flags.
pub fn isAboveMinimumRollout(flag: FeatureFlag, minimum: u64) bool {
    switch (flag.value) {
        .rollout_percentage => |pct| {
            return pct > minimum;
        },
        else => return false,
    }
}

/// Check if a percentage flag is at full rollout.
pub fn isFullRollout(flag: FeatureFlag) bool {
    switch (flag.value) {
        .rollout_percentage => |pct| {
            return pct >= 100;
        },
        else => return false,
    }
}

/// Check if a percentage flag has any rollout at all.
pub fn hasRollout(flag: FeatureFlag) bool {
    switch (flag.value) {
        .rollout_percentage => |pct| {
            return pct > 0;
        },
        else => return false,
    }
}

/// Format a flag summary into a buffer.
pub fn formatFlag(flag: FeatureFlag, buf: []u8) ![]u8 {
    var desc_str: []const u8 = "no description";
    if (flag.description) |d| {
        desc_str = d;
    }

    return std.fmt.bufPrint(buf, "{s}: {s} ({s})", .{
        flag.name,
        getFlagDisplayValue(flag),
        desc_str,
    });
}

// =============================================================================
// Tests (basic — these pass, but don't catch all the issues above)
// =============================================================================

test "create and check boolean flag" {
    const flag = createBooleanFlag("dark_mode", true, "Enable dark mode");
    try std.testing.expect(isFlagEnabled(flag, 12345));
}

test "create percentage flag" {
    const flag = createPercentageFlag("new_checkout", 50, null);
    // user_id 25 % 100 = 25, which is < 50, so enabled
    try std.testing.expect(isFlagEnabled(flag, 25));
    // user_id 75 % 100 = 75, which is >= 50, so disabled
    try std.testing.expect(!isFlagEnabled(flag, 75));
}

test "inactive flag is always disabled" {
    var flag = createBooleanFlag("test", true, null);
    flag.is_active = false;
    try std.testing.expect(!isFlagEnabled(flag, 0));
}

test "format flag with description" {
    const flag = createBooleanFlag("dark_mode", true, "Enable dark mode");
    var buf: [256]u8 = undefined;
    const result = try formatFlag(flag, &buf);
    try std.testing.expectEqualStrings("dark_mode: enabled (Enable dark mode)", result);
}
