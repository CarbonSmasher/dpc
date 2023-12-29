# === dpc:init === #
scoreboard objectives add _r dummy

# === moxlib:player/scroll/scrolled === #
function moxlib:player/scroll/direction
function #moxlib:api/player/on_scroll

# === test:main === #
execute store result score %rtest_main0 _r run data get entity @s SelectedItemSlot
scoreboard players operation @s moxlib.api.player.hotbar = %rtest_main0 _r
execute unless predicate moxlib:api/player/has_scrolled run scoreboard players set @s moxlib.api.player.scroll 0
execute if predicate moxlib:api/player/has_scrolled run function moxlib:player/scroll/scrolled
execute if score @s moxlib.api.player.hotbar > @s moxlib.player.hotbar.last run scoreboard players set @s moxlib.api.player.scroll 1
execute if score @s moxlib.api.player.hotbar < @s moxlib.player.hotbar.last run scoreboard players set @s moxlib.api.player.scroll -1
execute if score @s moxlib.api.player.hotbar matches ..2 run scoreboard players set @s moxlib.api.player.scroll 1
execute if score @s moxlib.api.player.hotbar matches 6.. run scoreboard players set @s moxlib.api.player.scroll -1
execute if score @s moxlib.api.player.hotbar = @s moxlib.player.hotbar.last run scoreboard players set @s moxlib.api.player.scroll 0
scoreboard players operation @s moxlib.player.hotbar.last = @s moxlib.api.player.hotbar
