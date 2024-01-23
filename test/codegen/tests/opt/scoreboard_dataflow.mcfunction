# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l5 _l 5
scoreboard players set %l8 _l 8
scoreboard players set %l10 _l 10

# === test:copy_elision === #
function test:copy_elision_fn

# === test:copy_elision_add === #
scoreboard players operation %atest_copy_elision_add.0 _r += %atest_copy_elision_add.1 _r
scoreboard players operation %Rtest_copy_elision_add.0 _r = %atest_copy_elision_add.0 _r

# === test:copy_elision_fn === #
scoreboard players operation %rtest_copy_elision_fn.0 _r = %atest_copy_elision_fn.0 _r
scoreboard players operation %atest_copy_elision_fn.0 _r *= %l5 _l
scoreboard players operation %rtest_copy_elision_fn.0 _r = %atest_copy_elision_fn.0 _r
scoreboard players operation %atest_copy_elision_fn.0 _r *= %l10 _l

# === test:copy_prop === #
scoreboard players operation %rtest_copy_prop.0 _r = @s foo
scoreboard players operation %rtest_copy_prop.1 _r = @s bar
scoreboard players operation %rtest_copy_prop.1 _r += %rtest_copy_prop.0 _r

# === test:main === #
scoreboard players set %rtest_main.0 _r 7
execute store result score %rtest_main.1 _r run scoreboard players operation %rtest_main.0 _r *= %l8 _l

# === test:op_to_cast === #
scoreboard players set %rtest_op_to_cast.0 _r 10
execute store result storage dpc:r rtest_op_to_cast_0 int 1 run scoreboard players add %rtest_op_to_cast.0 _r 8
