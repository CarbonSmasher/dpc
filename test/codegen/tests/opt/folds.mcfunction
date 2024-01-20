# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l1 _l 1
scoreboard players set %l2 _l 2

# === test:conditions === #
execute store success score %rtest_conditions.0 _r if predicate foo:bar

# === test:if_else === #
execute if predicate foo:bar run say Hello
execute unless predicate foo:bar run say Hello
execute if predicate foo:bar run say Hello

# === test:let_cond === #
scoreboard players operation %rtest_let_cond.0 _r = @s foo

# === test:main === #
scoreboard players operation %rtest_main.0 _r = foo bar
scoreboard players operation %rtest_main.0 _r = foo bar
scoreboard players operation %rtest_main.0 _r < foo bar
scoreboard players operation %rtest_main.0 _r > foo bar
scoreboard players operation %rtest_main.0 _r += foo bar
scoreboard players operation %rtest_main.0 _r -= foo bar
scoreboard players operation %rtest_main.0 _r *= foo bar
scoreboard players operation %rtest_main.0 _r *= foo bar
say Guaranteed
scoreboard players operation %rtest_main.0 _r *= %l2 _l
scoreboard players set %rtest_main.0 _r 0
scoreboard players operation %rtest_main.0 _r %= %l1 _l
scoreboard players set @s foo 1
scoreboard players set @s foo 0
data modify entity @s name.bar set value 7b
data modify storage dpc:r rtest_main_0 set from entity @s foo
execute store result entity foo bar float .7 run data get entity @s foo 9
