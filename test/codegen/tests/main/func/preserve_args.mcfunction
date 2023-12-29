# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l7 _l 7

# === test:main === #
scoreboard players set %rtest_main0 _r 56
scoreboard players set %rtest_main0 _r 70
scoreboard players operation %rtest_main1 _r = @s foo
scoreboard players operation %rtest_main2 _r = %rtest_main1 _r
scoreboard players operation %rtest_main2 _r *= %l7 _l
scoreboard players operation %rtest_main0 _r = %rtest_main2 _r
