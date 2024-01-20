# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l3 _l 3

# === fold:assign_const_add === #
scoreboard players operation %rfold_assign_const_add.0 _r = @s foo
scoreboard players operation %rfold_assign_const_add.1 _r = %rfold_assign_const_add.0 _r
scoreboard players operation %rfold_assign_const_add.1 _r = @s foo
scoreboard players operation %rfold_assign_const_add.0 _r = %rfold_assign_const_add.1 _r
scoreboard players add %rfold_assign_const_add.0 _r 10

# === fold:if_cond_assign === #
execute store success score %rfold_if_cond_assign.0 _r if biome ~ ~ ~ forest
execute store success score %rfold_if_cond_assign.0 _r unless predicate foo

# === fold:overwrite_op === #
scoreboard players set %rfold_overwrite_op.0 _r 6

# === fold:stack_peak === #
scoreboard players operation %rfold_stack_peak.0 _r = @s foo
scoreboard players operation %rfold_stack_peak.0 _r *= %l3 _l
scoreboard players remove %rfold_stack_peak.0 _r 4
scoreboard players operation %rfold_stack_peak.0 _r /= @s bar
scoreboard players add %rfold_stack_peak.0 _r 1

# === test:main === #
