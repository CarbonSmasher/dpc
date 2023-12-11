scoreboard players operation %r0 _r = .in math
scoreboard players set %r1 _r -400
scoreboard players operation %r0 _r %= %l3600 _l
execute if score %r0 _r matches 1800.. run scoreboard players set %r1 _r 400
execute store result score %r2 _r run scoreboard players operation %r0 _r %= %l1800 _l
scoreboard players remove %r2 _r 1800
scoreboard players operation %r2 _r *= %r0 _r
scoreboard players operation %r0 _r *= %r1 _r
scoreboard players add %r2 _r 4050000
scoreboard players operation %r0 _r /= %r2 _r
execute if score %r1 _r matches 400 run scoreboard players add %r0 _r 1
scoreboard players operation .out math = %r0 _r
