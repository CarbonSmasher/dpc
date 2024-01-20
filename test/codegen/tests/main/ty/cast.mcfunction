# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
execute store result storage dpc:r rtest_main_0 int 1 run scoreboard players get @s name
scoreboard players set %rtest_main.0 _r 7
scoreboard players operation %rtest_main.1 _r = %rtest_main.0 _r
scoreboard players operation %rtest_main.0 _r = @s bar
