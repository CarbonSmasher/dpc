# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l8 _l 8
scoreboard players set %l10 _l 10

# === test:func === #
scoreboard players operation %rtest_func0 _r = %atest_func0 _r
scoreboard players operation %rtest_func0 _r *= %l8 _l
scoreboard players remove %rtest_func0 _r 5
scoreboard players operation %rtest_func0 _r %= %l10 _l
# No not here
# Not after here
execute store success score %rtest_func0 _r if score %rtest_func0 _r matches 0
scoreboard players operation %rtest_func0 _r *= %rtest_func0 _r
scoreboard players operation %rtest_func1 _r = %rtest_func0 _r
scoreboard players operation %rtest_func0 _r *= %rtest_func1 _r
scoreboard players operation %rtest_func0 _r *= %rtest_func1 _r
scoreboard players operation %rtest_func0 _r *= %rtest_func1 _r
scoreboard players operation %rtest_func0 _r *= %rtest_func1 _r

# === test:main === #
scoreboard players set atest_func0 _r 5
function test:func
