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
execute if entity @s run say hi
execute if predicate foo:bar run say hi
execute if entity @s run say hi
execute if dimension minecraft:the_end run say hi
execute if biome ~ ~ 7 minecraft:forest run say hi
execute if loaded ~ ~ 7 run say hi
