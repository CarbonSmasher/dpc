######## main ########
# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l5 _l 5
scoreboard players set %l20 _l 20

# === items:ground_sweep/main === #
execute unless entity @s[tag=seent] run function items:ground_sweep/main_body_1
execute if score @s cooldown matches 1.. run function items:ground_sweep/main_body_2

# === items:ground_sweep/main_body_0 === #
execute store result score %ritems_ground_sweep_main4 _r run data get storage dpc:r sitems_ground_sweep_main0.rarity
execute if score %ritems_ground_sweep_main4 _r matches 0 run team join common @s
execute if score %ritems_ground_sweep_main4 _r matches 1 run team join uncommon @s
execute if score %ritems_ground_sweep_main4 _r matches 2 run team join rare @s
execute if score %ritems_ground_sweep_main4 _r matches 3 run team join epic @s
execute if score %ritems_ground_sweep_main4 _r matches 4 run team join legendary @s
execute if score %ritems_ground_sweep_main4 _r matches 5 run team join mythic @s
execute if score %ritems_ground_sweep_main4 _r matches 6 run team join supreme @s
execute if score %ritems_ground_sweep_main4 _r matches 7.. if score %ritems_ground_sweep_main4 _r matches ..8 run team join special @s
execute if score %ritems_ground_sweep_main4 _r matches 9.. run team join hydar @s
data merge entity @s {Glowing:1b}

# === items:ground_sweep/main_body_1 === #
data modify storage dpc:r sitems_ground_sweep_main0 set from entity @s Item.tag
execute store result score %ritems_ground_sweep_main1 _r run data get storage dpc:r sitems_ground_sweep_main0.clear
execute if score %ritems_ground_sweep_main1 _r matches 1 run kill
execute store result score %ritems_ground_sweep_main2 _r run data get storage dpc:r sitems_ground_sweep_main0.player_item
execute if score %ritems_ground_sweep_main2 _r matches 1 run kill
execute store result score %ritems_ground_sweep_main0 _r run data get storage dpc:r sitems_ground_sweep_main0.explorer_set
execute if score %ritems_ground_sweep_main0 _r matches 1 run function items:set/drop
execute store result score %ritems_ground_sweep_main3 _r run data get storage dpc:r sitems_ground_sweep_main0.wither_heart
execute if score %ritems_ground_sweep_main3 _r matches 1 run tag @s add wither_heart
execute if data storage dpc:r sitems_ground_sweep_main0.Upgrades.ench run tag @s add ench
execute if data storage dpc:r sitems_ground_sweep_main0.ench_book run tag @s add ench_book
execute if data storage dpc:r sitems_ground_sweep_main0.rarity run function items:ground_sweep/main_body_0
execute store result score %ritems_ground_sweep_main4 _r run data get storage dpc:r sitems_ground_sweep_main0.timer
execute if score %ritems_ground_sweep_main4 _r matches 1.. run data modify entity @s PickupDelay set value 32767
execute if data storage dpc:r sitems_ground_sweep_main0.coindata run function items:ground_sweep/coindata
tag @s add seent

# === items:ground_sweep/main_body_2 === #
scoreboard players remove @s cooldown 1
execute if score @s cooldown matches 0 run kill

# === player:hud/cooldown/cooldown_bar === #
scoreboard players set %rplayer_hud_cooldown_cooldown_bar0 _r 0
execute if score @s item.id matches 2 run scoreboard players operation %rplayer_hud_cooldown_cooldown_bar0 _r = @s cd.item.wrath
execute if score @s item.id matches 18 run scoreboard players operation %rplayer_hud_cooldown_cooldown_bar0 _r = @s cd.item.sdagger
execute if score @s item.id matches 39 run scoreboard players operation %rplayer_hud_cooldown_cooldown_bar0 _r = @s cd.item.gsword
execute if score @s item.id matches 52 run scoreboard players operation %rplayer_hud_cooldown_cooldown_bar0 _r = @s cd.item.cshard
execute if score @s item.id matches 69 run function player:hud/cooldown/cooldown_bar_body_0
execute store success score %rplayer_hud_cooldown_cooldown_bar1 _r if score %rplayer_hud_cooldown_cooldown_bar0 _r matches 1..
execute if score %rplayer_hud_cooldown_cooldown_bar1 _r matches 1 run function player:hud/cooldown/cooldown_bar_body_1
execute if score %rplayer_hud_cooldown_cooldown_bar1 _r matches 0 run function player:hud/cooldown/cooldown_bar_body_2
xp set @s 5 levels
scoreboard players operation %rplayer_hud_cooldown_cooldown_bar0 _r /= %l20 _l
execute if score %rplayer_hud_cooldown_cooldown_bar0 _r matches 1.. run scoreboard players add %rplayer_hud_cooldown_cooldown_bar0 _r 1
scoreboard players set %rplayer_hud_cooldown_cooldown_bar1 _r 1
execute if score @s item.id matches 2 run scoreboard players set %rplayer_hud_cooldown_cooldown_bar1 _r 30
execute if score @s item.id matches 18 run scoreboard players set %rplayer_hud_cooldown_cooldown_bar1 _r 10
execute if score @s item.id matches 39 run scoreboard players set %rplayer_hud_cooldown_cooldown_bar1 _r 2
execute if score @s item.id matches 52 run scoreboard players set %rplayer_hud_cooldown_cooldown_bar1 _r 15
scoreboard players operation %rplayer_hud_cooldown_cooldown_bar1 _r = %rplayer_hud_cooldown_cooldown_bar0 _r
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 1 run xp set @s 0 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 2 run xp set @s 1 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 3 run xp set @s 2 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 4 run xp set @s 3 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 5 run xp set @s 4 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 6 run xp set @s 5 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 7 run xp set @s 6 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 8 run xp set @s 7 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 9 run xp set @s 8 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 10 run xp set @s 9 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 11 run xp set @s 10 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 12 run xp set @s 11 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 13 run xp set @s 12 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 14 run xp set @s 13 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 15 run xp set @s 14 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 16 run xp set @s 15 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 17 run xp set @s 16 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 18 run xp set @s 17 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 19 run xp set @s 18 points
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 0 run function player:hud/cooldown/cooldown_bar_body_3
execute if score %rplayer_hud_cooldown_cooldown_bar2 _r matches 20 run scoreboard players reset %rplayer_hud_cooldown_cooldown_bar2
xp set @s 0 levels
scoreboard players operation aplayer_hud_cooldown_set_level0 _r = %rplayer_hud_cooldown_cooldown_bar1 _r
function player:hud/cooldown/set_level
execute if score @s item.id matches 69 run function items:abilities/berserk/chainsaw/cooldown

# === player:hud/cooldown/cooldown_bar_body_0 === #
scoreboard players operation %rplayer_hud_cooldown_cooldown_bar0 _r = @s cd.item.chainsaw
scoreboard players operation %rplayer_hud_cooldown_cooldown_bar0 _r /= %l5 _l

# === player:hud/cooldown/cooldown_bar_body_1 === #
scoreboard players operation %rplayer_hud_cooldown_cooldown_bar2 _r = %rplayer_hud_cooldown_cooldown_bar0 _r
scoreboard players operation %rplayer_hud_cooldown_cooldown_bar2 _r %= %l20 _l

# === player:hud/cooldown/cooldown_bar_body_2 === #
xp set @s 0 levels
xp set @s 0 points
scoreboard players reset %rplayer_hud_cooldown_cooldown_bar2

# === player:hud/cooldown/cooldown_bar_body_3 === #
xp set @s 9999 levels
xp set @s 89000 points

# === player:hud/cooldown/set_level === #
execute if score %aplayer_hud_cooldown_set_level0 _r matches 1 run xp set @s 1 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 2 run xp set @s 2 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 3 run xp set @s 3 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 4 run xp set @s 4 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 5 run xp set @s 5 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 6 run xp set @s 6 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 7 run xp set @s 7 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 8 run xp set @s 8 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 9 run xp set @s 9 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 10 run xp set @s 10 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 11 run xp set @s 11 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 12 run xp set @s 12 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 13 run xp set @s 13 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 14 run xp set @s 14 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 15 run xp set @s 15 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 16 run xp set @s 16 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 17 run xp set @s 17 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 18 run xp set @s 18 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 19 run xp set @s 19 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 20 run xp set @s 20 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 21 run xp set @s 21 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 22 run xp set @s 22 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 23 run xp set @s 23 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 24 run xp set @s 24 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 25 run xp set @s 25 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 26 run xp set @s 26 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 27 run xp set @s 27 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 28 run xp set @s 28 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 29 run xp set @s 29 levels
execute if score %aplayer_hud_cooldown_set_level0 _r matches 30 run xp set @s 30 levels

# === player:hud/end/main === #
data remove storage dungeons:items tempEndText

# === player:hud/refresh === #
function player:hud/cooldown/cooldown_bar
data remove storage dungeons:items tempCenterText
data merge storage dungeons:items {tempCenterText:'[{"score":{"name":"@s","objective":"stat.total.def"},"color":"green"},{"text":"❈ Defense   ","color":"green"}]'}
function player:hud/end/main
function player:hud/show_actionbar

# === player:main === #
execute if score @s cd.actionbar matches ..0 run function player:hud/refresh

# === test:main === #

######## opt ########
# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l5 _l 5
scoreboard players set %l20 _l 20

# === items:ground_sweep/main === #
execute unless entity @s[tag=seent] run function items:ground_sweep/main_body_1
execute if score @s cooldown matches 1.. run function items:ground_sweep/main_body_2

# === items:ground_sweep/main_body_0 === #
execute store result score %ritems_ground_sweep_main4 _r run data get storage dpc:r sitems_ground_sweep_main0.rarity
execute if score %ritems_ground_sweep_main4 _r matches 0 run team join common @s
execute if score %ritems_ground_sweep_main4 _r matches 1 run team join uncommon @s
execute if score %ritems_ground_sweep_main4 _r matches 2 run team join rare @s
execute if score %ritems_ground_sweep_main4 _r matches 3 run team join epic @s
execute if score %ritems_ground_sweep_main4 _r matches 4 run team join legendary @s
execute if score %ritems_ground_sweep_main4 _r matches 5 run team join mythic @s
execute if score %ritems_ground_sweep_main4 _r matches 6 run team join supreme @s
execute if score %ritems_ground_sweep_main4 _r matches 7..8 run team join special @s
execute if score %ritems_ground_sweep_main4 _r matches 9.. run team join hydar @s
data merge entity @s {Glowing:1b}

# === items:ground_sweep/main_body_1 === #
data modify storage dpc:r sitems_ground_sweep_main0 set from entity @s Item.tag
execute store result score %ritems_ground_sweep_main0 _r run data get storage dpc:r sitems_ground_sweep_main0.clear
execute if score %ritems_ground_sweep_main0 _r matches 1 run kill
execute store result score %ritems_ground_sweep_main1 _r run data get storage dpc:r sitems_ground_sweep_main0.player_item
execute if score %ritems_ground_sweep_main1 _r matches 1 run kill
execute store result score %ritems_ground_sweep_main2 _r run data get storage dpc:r sitems_ground_sweep_main0.explorer_set
execute if score %ritems_ground_sweep_main2 _r matches 1 run function items:set/drop
execute store result score %ritems_ground_sweep_main3 _r run data get storage dpc:r sitems_ground_sweep_main0.wither_heart
execute if score %ritems_ground_sweep_main3 _r matches 1 run tag @s add wither_heart
execute if data storage dpc:r sitems_ground_sweep_main0.Upgrades.ench run tag @s add ench
execute if data storage dpc:r sitems_ground_sweep_main0.ench_book run tag @s add ench_book
execute if data storage dpc:r sitems_ground_sweep_main0.rarity run function items:ground_sweep/main_body_0
execute store result score %ritems_ground_sweep_main4 _r run data get storage dpc:r sitems_ground_sweep_main0.timer
execute if score %ritems_ground_sweep_main4 _r matches 1.. run data modify entity @s PickupDelay set value 32767
execute if data storage dpc:r sitems_ground_sweep_main0.coindata run function items:ground_sweep/coindata
tag @s add seent

# === items:ground_sweep/main_body_2 === #
scoreboard players remove @s cooldown 1
execute if score @s cooldown matches 0 run kill

# === player:hud/refresh === #
scoreboard players set %rplayer_hud_refresh3 _r 0
execute if score @s item.id matches 2 run scoreboard players operation %rplayer_hud_refresh3 _r = @s cd.item.wrath
execute if score @s item.id matches 18 run scoreboard players operation %rplayer_hud_refresh3 _r = @s cd.item.sdagger
execute if score @s item.id matches 39 run scoreboard players operation %rplayer_hud_refresh3 _r = @s cd.item.gsword
execute if score @s item.id matches 52 run scoreboard players operation %rplayer_hud_refresh3 _r = @s cd.item.cshard
execute if score @s item.id matches 69 run function player:hud/refresh_body_0
execute store success score %rplayer_hud_refresh2 _r if score %rplayer_hud_refresh3 _r matches 1..
execute if score %rplayer_hud_refresh2 _r matches 1 run function player:hud/refresh_body_1
execute if score %rplayer_hud_refresh2 _r matches 0 run function player:hud/refresh_body_2
xp set @s 5 levels
scoreboard players operation %rplayer_hud_refresh3 _r /= %l20 _l
execute if score %rplayer_hud_refresh3 _r matches 1.. store result score %rplayer_hud_refresh2 _r run scoreboard players add %rplayer_hud_refresh3 _r 1
scoreboard players set %rplayer_hud_refresh3 _r 1
execute if score @s item.id matches 2 run scoreboard players set %rplayer_hud_refresh3 _r 30
execute if score @s item.id matches 18 run scoreboard players set %rplayer_hud_refresh3 _r 10
execute if score @s item.id matches 39 run scoreboard players set %rplayer_hud_refresh3 _r 2
execute if score @s item.id matches 52 run scoreboard players set %rplayer_hud_refresh3 _r 15
execute if score %rplayer_hud_refresh1 _r matches 1 run xp set @s 0 points
execute if score %rplayer_hud_refresh1 _r matches 2 run xp set @s 1 points
execute if score %rplayer_hud_refresh1 _r matches 3 run xp set @s 2 points
execute if score %rplayer_hud_refresh1 _r matches 4 run xp set @s 3 points
execute if score %rplayer_hud_refresh1 _r matches 5 run xp set @s 4 points
execute if score %rplayer_hud_refresh1 _r matches 6 run xp set @s 5 points
execute if score %rplayer_hud_refresh1 _r matches 7 run xp set @s 6 points
execute if score %rplayer_hud_refresh1 _r matches 8 run xp set @s 7 points
execute if score %rplayer_hud_refresh1 _r matches 9 run xp set @s 8 points
execute if score %rplayer_hud_refresh1 _r matches 10 run xp set @s 9 points
execute if score %rplayer_hud_refresh1 _r matches 11 run xp set @s 10 points
execute if score %rplayer_hud_refresh1 _r matches 12 run xp set @s 11 points
execute if score %rplayer_hud_refresh1 _r matches 13 run xp set @s 12 points
execute if score %rplayer_hud_refresh1 _r matches 14 run xp set @s 13 points
execute if score %rplayer_hud_refresh1 _r matches 15 run xp set @s 14 points
execute if score %rplayer_hud_refresh1 _r matches 16 run xp set @s 15 points
execute if score %rplayer_hud_refresh1 _r matches 17 run xp set @s 16 points
execute if score %rplayer_hud_refresh1 _r matches 18 run xp set @s 17 points
execute if score %rplayer_hud_refresh1 _r matches 19 run xp set @s 18 points
execute if score %rplayer_hud_refresh1 _r matches 0 run function player:hud/refresh_body_3
execute if score %rplayer_hud_refresh1 _r matches 20 run scoreboard players reset %rplayer_hud_refresh1
xp set @s 0 levels
scoreboard players operation %rplayer_hud_refresh1 _r = %rplayer_hud_refresh2 _r
execute if score %rplayer_hud_refresh1 _r matches 1 run xp set @s 1 levels
execute if score %rplayer_hud_refresh1 _r matches 2 run xp set @s 2 levels
execute if score %rplayer_hud_refresh1 _r matches 3 run xp set @s 3 levels
execute if score %rplayer_hud_refresh1 _r matches 4 run xp set @s 4 levels
execute if score %rplayer_hud_refresh1 _r matches 5 run xp set @s 5 levels
execute if score %rplayer_hud_refresh1 _r matches 6 run xp set @s 6 levels
execute if score %rplayer_hud_refresh1 _r matches 7 run xp set @s 7 levels
execute if score %rplayer_hud_refresh1 _r matches 8 run xp set @s 8 levels
execute if score %rplayer_hud_refresh1 _r matches 9 run xp set @s 9 levels
execute if score %rplayer_hud_refresh1 _r matches 10 run xp set @s 10 levels
execute if score %rplayer_hud_refresh1 _r matches 11 run xp set @s 11 levels
execute if score %rplayer_hud_refresh1 _r matches 12 run xp set @s 12 levels
execute if score %rplayer_hud_refresh1 _r matches 13 run xp set @s 13 levels
execute if score %rplayer_hud_refresh1 _r matches 14 run xp set @s 14 levels
execute if score %rplayer_hud_refresh1 _r matches 15 run xp set @s 15 levels
execute if score %rplayer_hud_refresh1 _r matches 16 run xp set @s 16 levels
execute if score %rplayer_hud_refresh1 _r matches 17 run xp set @s 17 levels
execute if score %rplayer_hud_refresh1 _r matches 18 run xp set @s 18 levels
execute if score %rplayer_hud_refresh1 _r matches 19 run xp set @s 19 levels
execute if score %rplayer_hud_refresh1 _r matches 20 run xp set @s 20 levels
execute if score %rplayer_hud_refresh1 _r matches 21 run xp set @s 21 levels
execute if score %rplayer_hud_refresh1 _r matches 22 run xp set @s 22 levels
execute if score %rplayer_hud_refresh1 _r matches 23 run xp set @s 23 levels
execute if score %rplayer_hud_refresh1 _r matches 24 run xp set @s 24 levels
execute if score %rplayer_hud_refresh1 _r matches 25 run xp set @s 25 levels
execute if score %rplayer_hud_refresh1 _r matches 26 run xp set @s 26 levels
execute if score %rplayer_hud_refresh1 _r matches 27 run xp set @s 27 levels
execute if score %rplayer_hud_refresh1 _r matches 28 run xp set @s 28 levels
execute if score %rplayer_hud_refresh1 _r matches 29 run xp set @s 29 levels
execute if score %rplayer_hud_refresh1 _r matches 30 run xp set @s 30 levels
execute if score @s item.id matches 69 run function items:abilities/berserk/chainsaw/cooldown
data remove storage dungeons:items tempCenterText
data merge storage dungeons:items {tempCenterText:'[{"score":{"name":"@s","objective":"stat.total.def"},"color":"green"},{"text":"❈ Defense   ","color":"green"}]'}
data remove storage dungeons:items tempEndText
function player:hud/show_actionbar

# === player:hud/refresh_body_0 === #
scoreboard players operation %rplayer_hud_refresh3 _r = @s cd.item.chainsaw
scoreboard players operation %rplayer_hud_refresh3 _r /= %l5 _l

# === player:hud/refresh_body_1 === #
scoreboard players operation %rplayer_hud_refresh1 _r = %rplayer_hud_refresh3 _r
scoreboard players operation %rplayer_hud_refresh1 _r %= %l20 _l

# === player:hud/refresh_body_2 === #
xp set @s 0 levels
xp set @s 0 points
scoreboard players reset %rplayer_hud_refresh1

# === player:hud/refresh_body_3 === #
xp set @s 9999 levels
xp set @s 89000 points

# === player:main === #
execute if score @s cd.actionbar matches ..0 run function player:hud/refresh

# === test:main === #
