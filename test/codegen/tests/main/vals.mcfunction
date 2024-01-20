# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
data modify storage dpc:r rtest_main_0 set from entity foo bar
scoreboard players operation %rtest_main.0 _r = foo bar
data remove storage dpc:r rtest_main_0
scoreboard players reset %rtest_main.0
data modify storage dpc:r rtest_main_0 set from entity @s foo
scoreboard players operation %rtest_main.0 _r = @s bar
data modify storage dpc:r rtest_main_0 set from storage foo:bar bar
data modify storage dpc:r rtest_main_0 set from storage foo:bar bar[6]
data modify storage dpc:r rtest_main_0 set from storage foo:bar foo[5].name
data modify storage dpc:r rtest_main_1 set from storage dpc:r rtest_main_0[7]
data modify storage dpc:r rtest_main_0 set from storage foo:bar
execute store result score %rtest_main.0 _r run data get storage foo:bar baz
execute store result storage dpc:r rtest_main_0 int 1 run scoreboard players get @r name
