# === dpc:init ===
scoreboard objectives add _r dummy

# === test:main ===
execute store result entity @s foo int .8 run say hello
scoreboard players set %rtest_main0 _r 7
execute store success score %rtest_main0 _r run data get entity baz foo 4.2
