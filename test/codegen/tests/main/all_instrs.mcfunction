scoreboard players set %rtest_main0 _r 7
scoreboard players operation %rtest_main0 _r = %rtest_main0 _r
scoreboard players add %rtest_main0 _r 1
scoreboard players remove %rtest_main0 _r 1
scoreboard players operation %rtest_main0 _r *= %l1 _l
scoreboard players operation %rtest_main0 _r /= %l1 _l
scoreboard players operation %rtest_main0 _r %= %l1 _l
scoreboard players operation %rtest_main0 _r < %l1 _l
scoreboard players operation %rtest_main0 _r > %l1 _l
scoreboard players set %rtest_main1 _r 3
scoreboard players operation %rtest_main0 _r >< %rtest_main1 _r
scoreboard players reset %rtest_main0
execute if score %rtest_main0 _r matches ..-1 run scoreboard players operation %rtest_main0 _r *= %l-1 _l
scoreboard players operation %rtest_main1 _r = %rtest_main0 _r
scoreboard players operation %rtest_main0 _r *= %rtest_main1 _r
scoreboard players operation %rtest_main0 _r *= %rtest_main1 _r
scoreboard players operation %rtest_main0 _r *= %rtest_main0 _r
scoreboard players operation %rtest_main0 _r *= %rtest_main0 _r
scoreboard players get %rtest_main0 _r
say foo
me foo
tm foo
banlist
ban-ip foo
ban-ip foo bar
pardon-ip foo
whitelist on
whitelist off
whitelist reload
whitelist list
list
publish
reload
seed
stop
stopsound
difficulty
difficulty hard
spectate
w foo bar
kill foo
enchant foo minecraft:power 5
xp set foo 6 levels
xp set foo 98 points
xp add foo 45 levels
xp query foo points
tag foo add bar
tag foo remove bar
tag foo list
ride foo mount bar
ride foo dismount
spectate foo bar
scoreboard objectives remove foo
scoreboard objectives list
trigger foo add 6
trigger foo set 0
datapack disable foo
datapack enable foo
list uuids
scoreboard objectives add foo dummy
scoreboard objectives add bar foo.bar
setworldspawn 0 ~6 8 ~3
summon minecraft:zombie ~ ~ ~ {CustomName:"Foo"}
summon minecraft:zombie
scoreboard players set atest_foo0 _r 7
data modify storage dpc:r atest_foo1 set value 82
function test:foo
function #minecraft:tick
give @s stick{foo:3b} 6
give @s stick
effect clear
effect clear @s speed
effect give @s speed 4 8 true
effect give @s speed 4 8
effect give @s speed 4
effect give @s speed
effect give @s speed infinite
effect give @s speed infinite 1 true
time set 85
time add 4d
# This does something
execute store result score #foo bar run data get entity @s Health
clear
spawnpoint @r ~ ~ ~5
tp @a @s
tp @e
tp 0.0 0.0 ~
tp @s 0.0 0.0 0.0 0.4 ~4.0
tp @s 0.0 0.0 0.0 facing 0.4 ~4.0 7.4
tp @s 0.0 0.0 0.0 facing @p
