# === dpc:init === #
scoreboard objectives add _r dummy

# === fold:let_cond_not === #
execute store success score %rfold_let_cond_not.0 _r unless score @s foo matches 10

# === fold:let_cond_prop === #
execute unless score @s foo matches 7 run say hello
execute if predicate foo:bar unless predicate bar:foo run say hello

# === fold:manual_or === #
scoreboard players operation %rfold_manual_or.0 _r = @s foo
scoreboard players operation %rfold_manual_or.1 _r = @s bar
execute if score %rfold_manual_or.1 _r matches 1 run scoreboard players set %rfold_manual_or.0 _r 1

# === test:main === #
