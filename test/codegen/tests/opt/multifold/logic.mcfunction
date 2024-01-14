# === dpc:init === #
scoreboard objectives add _r dummy

# === fold:assign_if_bool === #
execute store success score %rfold_assign_if_bool0 _r if score @s foo matches 7
execute unless score %rfold_assign_if_bool0 _r matches 1 run say hello

# === test:main === #
