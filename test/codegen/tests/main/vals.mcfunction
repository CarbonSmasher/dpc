# === dpc:init === #
scoreboard objectives add _r dummy
data merge storage dpc:r {}

# === test:main === #
data modify storage dpc:r stest_main0 set from entity foo bar
scoreboard players operation %rtest_main0 _r = foo bar
data remove storage dpc:r stest_main0
scoreboard players reset %rtest_main0
data modify storage dpc:r stest_main0 set from entity @s foo
scoreboard players operation %rtest_main0 _r = @s bar
data modify storage dpc:r stest_main0 set from storage foo:bar bar
data modify storage dpc:r stest_main0 set from storage foo:bar bar[6]
data modify storage dpc:r stest_main0 set from storage foo:bar foo[5].name
data modify storage dpc:r stest_main1 set from storage dpc:r stest_main0[7]
data modify storage dpc:r stest_main0 set from storage foo:bar
execute store result score %rtest_main0 _r run data get storage foo:bar baz
execute store result storage dpc:r stest_main0 int 1 run scoreboard players get @r name
