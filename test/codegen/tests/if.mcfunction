scoreboard players set %r0 _r 7
execute if score %r0 _r matches 7 run say hello
execute if score %r1 _r matches ..2147483647 unless score %r1 _r matches 5 run scoreboard players set %r0 _r 3
