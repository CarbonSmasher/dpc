# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l8 _l 8

# === test:main === #
scoreboard players set %rtest_main0 _r 7
execute store result score %rtest_main1 _r run scoreboard players operation %rtest_main0 _r *= %l8 _l
