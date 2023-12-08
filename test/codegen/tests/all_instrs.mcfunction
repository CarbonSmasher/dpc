scoreboard players set %r0 _r 7
scoreboard players operation %r0 _r = %r0 _r
scoreboard players add %r0 _r 1
scoreboard players remove %r0 _r 1
scoreboard players operation %r0 _r *= %l1 _l
scoreboard players operation %r0 _r /= %l1 _l
scoreboard players operation %r0 _r %= %l1 _l
scoreboard players operation %r0 _r < %l1 _l
scoreboard players operation %r0 _r > %l1 _l
scoreboard players set %r1 _r 3
scoreboard players operation %r0 _r >< %r1 _r
execute if score %r0 _r matches ..-1 run scoreboard players operation %r0 _r *= %l-1 _l
scoreboard players operation %r1 _r = %r0 _r
scoreboard players operation %r0 _r *= %r1 _r
scoreboard players operation %r0 _r *= %r1 _r
scoreboard players operation %r0 _r *= %r0 _r
scoreboard players operation %r0 _r *= %r0 _r
scoreboard players get %r0 _r
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
