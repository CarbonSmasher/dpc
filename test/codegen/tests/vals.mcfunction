data modify storage dpc:r stest_main0 set from entity foo bar
scoreboard players operation %rtest_main0 _r = foo bar
data remove storage dpc:r stest_main0
scoreboard players reset %rtest_main0
data modify storage dpc:r stest_main0 set from entity @s foo
scoreboard players operation %rtest_main0 _r = @s bar
data modify storage dpc:r stest_main0 set from storage foo:bar bar
data modify storage dpc:r stest_main0 set from storage foo:bar bar[6]
data modify storage dpc:r stest_main0 set from storage foo:bar foo[5].name
execute store result score %rtest_main0 _r run data get storage foo:bar baz
execute store result storage dpc:r stest_main0 run scoreboard players get @r name
