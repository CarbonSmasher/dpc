# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l6 _l 6

# === test:div_by_self === #
scoreboard players set %rtest_div_by_self0 _r 1
scoreboard players set %rtest_div_by_self0 _r 0

# === test:main === #
scoreboard players set %rtest_main0 _r 12
scoreboard players set %rtest_main0 _r 10
scoreboard players operation %rtest_main0 _r *= %l6 _l
scoreboard players set %rtest_main0 _r 18
scoreboard players set %rtest_main0 _r 18

# === test:setblock === #
setblock ~ ~ ~ smooth_stone

# === test:zero_mul === #
scoreboard players set %rtest_zero_mul0 _r 0
