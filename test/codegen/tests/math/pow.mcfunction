# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main.0 _r 8
# Should set to one
scoreboard players set %rtest_main.0 _r 1
# Should generate nothing
# Should multiply by self
scoreboard players operation %rtest_main.0 _r *= %rtest_main.0 _r
# Should generate 3 instructions
scoreboard players operation %rtest_main.1 _r = %rtest_main.0 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
# Should generate many multiplications
scoreboard players operation %rtest_main.1 _r = %rtest_main.0 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
# Should generate a couple self multiplications and some multiplications after that
scoreboard players operation %rtest_main.0 _r *= %rtest_main.0 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.0 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.0 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.0 _r
scoreboard players operation %rtest_main.1 _r = %rtest_main.0 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
scoreboard players operation %rtest_main.0 _r *= %rtest_main.1 _r
