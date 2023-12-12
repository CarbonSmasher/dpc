data modify storage dpc:r stest_main0 set from entity foo bar
scoreboard players operation %rtest_main0 _r = foo bar
data remove storage dpc:r stest_main0
scoreboard players reset %rtest_main0 _r
data modify storage dpc:r stest_main0 set from entity @s foo
scoreboard players operation %rtest_main0 _r = @s bar
data modify storage dpc:r stest_main0 set from storage foo:bar bar
