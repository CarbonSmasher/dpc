# === dpc:init === #
scoreboard objectives add _r dummy
data merge storage dpc:r {}
scoreboard objectives add _l dummy
scoreboard players set %l2 _l 2

# === test:main === #
scoreboard players operation %rtest_main0 _r = foo bar
scoreboard players operation %rtest_main0 _r = foo bar
scoreboard players operation %rtest_main0 _r < foo bar
scoreboard players operation %rtest_main0 _r > foo bar
scoreboard players operation %rtest_main0 _r += foo bar
scoreboard players operation %rtest_main0 _r -= foo bar
scoreboard players operation %rtest_main0 _r *= foo bar
scoreboard players operation %rtest_main0 _r *= foo bar
scoreboard players operation %rtest_main0 _r *= %l2 _l
scoreboard players set %rtest_main0 _r 0
data modify entity @s name.bar set value 7b
data modify storage dpc:r stest_main0 set from entity @s foo
execute store result entity foo bar float .7 run data get entity @s foo 9
