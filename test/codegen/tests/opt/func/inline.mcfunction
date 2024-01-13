# === test:dont_inline === #
say Don't inline me

# === test:main === #
say Inline me
function test:dont_inline
execute if predicate test:test run say Inline me
execute if predicate test:test run function test:modifier_dont_inline
execute as @s at @s run function test:modifier_inline

# === test:modifier_dont_inline === #
say Foo
say Bar

# === test:modifier_inline === #
say Inline me
