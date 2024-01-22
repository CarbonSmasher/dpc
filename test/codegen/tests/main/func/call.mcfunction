# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l3 _l 3
scoreboard players set %l6 _l 6
scoreboard players set %l10 _l 10

# === test:fn === #
scoreboard players operation %atest_fn.0 _r *= %atest_fn.1 _r
data modify entity @s DisplayName set from storage dpc:r atest_fn_2

# === test:fn2 === #
scoreboard players operation %rtest_fn2.0 _r = %atest_fn2.0 _r
scoreboard players add %atest_fn2.0 _r 3
scoreboard players operation %atest_fn2.0 _r /= %l3 _l

# === test:main === #
scoreboard players set %atest_fn.1 _r -44
scoreboard players set %atest_fn.0 _r 7
data modify storage dpc:r atest_fn_2 set value "foo"
function test:fn
scoreboard players operation %rtest_main.0 _r = %Rtest_fn.0 _r
scoreboard players operation %atest_fn2.0 _r = @s foo
scoreboard players operation %atest_fn2.0 _r *= %l10 _l
scoreboard players operation %atest_fn2.0 _r %= %l6 _l
function test:fn2
