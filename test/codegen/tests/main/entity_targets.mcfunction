# === dpc:init === #
scoreboard objectives add _l dummy
scoreboard players set %l8 _l 8

# === test:main === #
data modify entity foo name set value 7f
scoreboard players operation @s[type=wolf,tag=bar,predicate=foo:bar] foo *= %l8 _l
data modify entity @r[nbt={foo:7b},limit=1] name[7] set value "bar"
