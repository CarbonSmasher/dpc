# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
execute store result storage dpc:r rtest_main0 int 1 run scoreboard players get @s name
scoreboard players set %rtest_main0 _r 7
scoreboard players operation %rtest_main1 _r = %rtest_main0 _r
scoreboard players operation %rtest_main0 _r = @s bar
