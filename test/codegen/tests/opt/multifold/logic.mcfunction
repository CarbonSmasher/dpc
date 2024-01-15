# === dpc:init === #
scoreboard objectives add _r dummy

# === fold:let_cond_not === #
execute store success score %rfold_let_cond_not0 _r unless score @s foo matches 10

# === fold:let_cond_prop === #
execute unless score @s foo matches 7 run say hello
execute if predicate foo:bar unless predicate bar:foo run say hello
execute unless predicate foo:bar if predicate foo:bar run say hello

# === test:main === #
