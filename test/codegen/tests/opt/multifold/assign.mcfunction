# === dpc:init === #
scoreboard objectives add _r dummy

# === fold:assign_const_add === #
scoreboard players operation %rfold_assign_const_add0 _r = @s foo
scoreboard players operation %rfold_assign_const_add1 _r = %rfold_assign_const_add0 _r
scoreboard players operation %rfold_assign_const_add1 _r = @s foo
scoreboard players operation %rfold_assign_const_add0 _r = %rfold_assign_const_add1 _r
scoreboard players add %rfold_assign_const_add0 _r 10

# === fold:if_cond_assign === #
execute store success score %rfold_if_cond_assign0 _r if biome ~ ~ ~ forest
execute store success score %rfold_if_cond_assign0 _r unless predicate foo

# === fold:overwrite_op === #
scoreboard players set %rfold_overwrite_op0 _r 6

# === test:main === #
