# === dpc:init ===
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l6 _l 6

# === test:main ===
scoreboard players set %rtest_main0 _r 12
scoreboard players set %rtest_main0 _r 10
scoreboard players operation %rtest_main0 _r *= %l6 _l
scoreboard players set %rtest_main0 _r 18
scoreboard players set %rtest_main0 _r 18
