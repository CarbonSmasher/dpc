# === dpc:init === #
scoreboard objectives add _l dummy
scoreboard players set %l0 _l 0
scoreboard players set %l1 _l 1

# === test:main === #
execute if score %l1 _l matches 1 run say hi
execute if score %l1 _l matches 2.. run say hi
execute if score %l1 _l matches 1.. run say hi
execute if score %l1 _l matches ..0 run say hi
execute if score %l1 _l matches ..1 run say hi
execute if score %l0 _l matches 0 run say hi
execute if score %l1 _l matches 1 run say hi
execute unless score %l1 _l matches 1 run say hi
execute if score %l1 _l matches 1 run say hi
execute unless score %l1 _l matches 1 run say hi
execute if predicate bar:foo if predicate foo:bar run say hi
execute if entity @s run say hi
execute if predicate foo:bar run say hi
execute if entity @s run say hi
execute if dimension minecraft:the_end run say hi
execute if biome ~ ~ 7 minecraft:forest run say hi
execute if loaded ~ ~ 7 run say hi
