# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main.0 _r 0
scoreboard players set %rtest_main.0 _r 2
execute if predicate foo:bar run scoreboard players add %rtest_main.0 _r 1
scoreboard players set %rtest_main.0 _r 0
scoreboard players set %rtest_main.1 _r 8
execute if predicate foo:bar run function test:main_body_0

# === test:main_body_0 === #
scoreboard players operation %rtest_main.1 _r %= %rtest_main.0 _r
scoreboard players set %rtest_main.2 _r 10
