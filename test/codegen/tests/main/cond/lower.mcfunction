# === dpc:init === #
scoreboard objectives add _r dummy

# === test:bool_cond === #
execute if score @s foo matches 1 run say hello
execute if score @s foo matches 0 run say goodbye

# === test:main === #

# === test:not_and === #
execute store success score %rtest_not_and.0 _r unless predicate bar:foo
execute unless predicate foo:bar run scoreboard players add %rtest_not_and.0 _r 1
execute if score %rtest_not_and.0 _r matches 1.. run say Hello

# === test:or === #
execute store success score %rtest_or.0 _r if predicate bar:foo
execute if predicate foo:bar run scoreboard players add %rtest_or.0 _r 1
execute if score %rtest_or.0 _r matches 1.. run say Hello
execute store success score %rtest_or.0 _r if function test:or_body_0
execute if predicate foo:bar4 run scoreboard players add %rtest_or.0 _r 1
execute if score %rtest_or.0 _r matches 1.. run say Hello2
execute if data storage foo:bar foo run function test:or_body_1
execute store success score %rtest_or.0 _r if score @s foo matches 1
scoreboard players operation %rtest_or.0 _r += @s bar
scoreboard players operation %rtest_or.1 _r = %rtest_or.0 _r
scoreboard players operation %rtest_or.1 _r += @s foo
execute if score %rtest_or.1 _r matches 1.. run say Hello5

# === test:or_body_0 === #
execute if predicate foo:bar run return 1
execute if predicate foo:bar2 if predicate foo:bar3 run return 1

# === test:or_body_1 === #
say Hello3
say Hello4

# === test:xor === #
execute store success score %rtest_xor.0 _r if predicate foo:bar
execute if predicate bar:foo run scoreboard players remove %rtest_xor.0 _r 1
execute unless score %rtest_xor.0 _r matches 0 run say Hello
