# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main0 _r 0
scoreboard players set %rtest_main0 _r 2
execute if predicate foo:bar run scoreboard players add %rtest_main0 _r 1
scoreboard players set %rtest_main0 _r 0
scoreboard players set %rtest_main1 _r 8
execute if predicate foo:bar run function test:main_body_0

# === test:main_body_0 === #
scoreboard players operation %rtest_main1 _r %= %rtest_main0 _r
scoreboard players set %rtest_main2 _r 10
