# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #

# === test:partial === #
execute if predicate foo:bar run say hello

# === test:prop === #
execute if score foo bar matches 5.. run say hello
scoreboard players set %rtest_prop.0 _r 10
scoreboard players operation %rtest_prop.1 _r = @s foo
execute if score %rtest_prop.1 _r matches 15 run scoreboard players add %rtest_prop.0 _r 15
scoreboard players operation %rtest_prop.1 _r = @s bar
scoreboard players operation %rtest_prop.0 _r += %rtest_prop.1 _r
scoreboard players set %rtest_prop.1 _r 1
scoreboard players set %rtest_prop.1 _r 0
