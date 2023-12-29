# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l8 _l 8

# === test:main === #
scoreboard players set %rtest_main0 _r 7
scoreboard players operation %rtest_main1 _r = %rtest_main0 _r
scoreboard players operation %rtest_main0 _r *= %l8 _l
scoreboard players operation %rtest_main1 _r += %rtest_main0 _r
scoreboard players operation %rtest_main0 _r >< %rtest_main1 _r
scoreboard players operation %rtest_main2 _r = %rtest_main1 _r
scoreboard players operation %rtest_main1 _r = %rtest_main2 _r
scoreboard players operation %rtest_main2 _r *= %rtest_main1 _r
scoreboard players operation %rtest_main2 _r *= %rtest_main1 _r
scoreboard players operation %rtest_main2 _r *= %rtest_main1 _r
scoreboard players operation %rtest_main0 _r = %rtest_main2 _r
