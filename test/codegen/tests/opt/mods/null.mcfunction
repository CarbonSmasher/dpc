# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main.0 _r 0
scoreboard players add %rtest_main.0 _r 2
execute as @e run say hello
execute as @e[type=zombie] as @e[type=wolf] run custom command
