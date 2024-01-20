# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l7 _l 7

# === test:main === #
scoreboard players set %rtest_main.0 _r 56
scoreboard players set %rtest_main.0 _r 70
scoreboard players operation %rtest_main.1 _r = @s foo
scoreboard players operation %rtest_main.1 _r *= %l7 _l
scoreboard players operation %rtest_main.0 _r = %rtest_main.1 _r
