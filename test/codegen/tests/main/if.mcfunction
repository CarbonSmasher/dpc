# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l5 _l 5

# === test:main === #
scoreboard players set %rtest_main.0 _r 7
execute if score %rtest_main.0 _r matches 7 run say hello
execute if score %rtest_main.1 _r matches ..2147483647 unless score %l5 _l = %rtest_main.1 _r run scoreboard players set %rtest_main.0 _r 3
execute store success score %rtest_main.1 _r if predicate foo:bar
execute if score %rtest_main.1 _r matches 1 run say True
execute if score %rtest_main.1 _r matches 0 run say False
