# === dpc:init ===
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l2 _l 2

# === test:main ===
scoreboard players operation %rtest_main0 _r = foo bar
scoreboard players operation %rtest_main0 _r = foo bar
scoreboard players operation %rtest_main0 _r < foo bar
scoreboard players operation %rtest_main0 _r > foo bar
scoreboard players operation %rtest_main0 _r *= %l2 _l
scoreboard players set %rtest_main0 _r 0
