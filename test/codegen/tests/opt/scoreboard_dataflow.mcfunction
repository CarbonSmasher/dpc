# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l8 _l 8

# === test:copy_prop === #
scoreboard players operation %rtest_copy_prop0 _r = @s foo
scoreboard players operation %rtest_copy_prop1 _r = @s bar
scoreboard players operation %rtest_copy_prop1 _r += %rtest_copy_prop0 _r

# === test:main === #
scoreboard players set %rtest_main0 _r 7
execute store result score %rtest_main1 _r run scoreboard players operation %rtest_main0 _r *= %l8 _l

# === test:op_to_cast === #
scoreboard players set %rtest_op_to_cast0 _r 10
execute store result storage dpc:r stest_op_to_cast0 int 1 run scoreboard players add %rtest_op_to_cast0 _r 8
