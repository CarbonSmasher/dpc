# === dpc:init === #
scoreboard objectives add _r dummy

# === test:fn === #
scoreboard players operation %rtest_fn.0 _r = %atest_fn.0 _r
scoreboard players operation %rtest_fn.0 _r *= %atest_fn.1 _r
data modify entity @s DisplayName set from storage dpc:r atest_fn_2

# === test:main === #
scoreboard players set %rtest_main.0 _r -44
scoreboard players set %atest_fn.0 _r 7
scoreboard players operation %atest_fn.1 _r = %rtest_main.0 _r
data modify storage dpc:r atest_fn_2 set value "foo"
function test:fn
scoreboard players operation %rtest_main.0 _r = %Rtest_fn.0 _r
