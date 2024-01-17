# === dpc:init === #
scoreboard objectives add _r dummy

# === test:bool_cond === #
execute if score @s foo matches 1 run say hello
execute if score @s foo matches 0 run say goodbye

# === test:main === #

# === test:not_and === #
execute store success score %rtest_not_and0 _r unless predicate bar:foo
execute unless predicate foo:bar run scoreboard players add %rtest_not_and0 _r 1
execute if score %rtest_not_and0 _r matches 1.. run say Hello

# === test:or === #
execute store success score %rtest_or0 _r if predicate bar:foo
execute if predicate foo:bar run scoreboard players add %rtest_or0 _r 1
execute if score %rtest_or0 _r matches 1.. run say Hello
execute store success score %rtest_or0 _r if predicate foo:bar2 if predicate foo:bar3
execute if predicate foo:bar run scoreboard players add %rtest_or0 _r 1
execute store success score %rtest_or1 _r if predicate foo:bar4
execute if score %rtest_or0 _r matches 1.. run scoreboard players add %rtest_or1 _r 1
execute if score %rtest_or1 _r matches 1.. run say Hello2
