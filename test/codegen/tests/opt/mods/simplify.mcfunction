# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main.0 _r 0
execute if score %rtest_main.0 _r matches ..2147483647 run say hello
execute if score %rtest_main.0 _r matches 1 run say hello
say guaranteed 1
say guaranteed 2
say guaranteed 3
say guaranteed 4
say hello
say hello
