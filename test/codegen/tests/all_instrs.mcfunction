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
scoreboard players reset %rtest_main0 _r
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