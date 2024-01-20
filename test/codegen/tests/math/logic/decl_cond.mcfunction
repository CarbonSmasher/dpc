# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main.0 _r 120
scoreboard players set %rtest_main.1 _r 991
execute store success score %rtest_main.2 _r if score %rtest_main.0 _r = %rtest_main.1 _r
execute store success score %rtest_main.2 _r unless score %rtest_main.0 _r = %rtest_main.1 _r
