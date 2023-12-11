scoreboard players set %r0 _r 0
execute if score %r0 _r matches ..2147483647 run say hello
execute if score %r0 _r matches 1 run say hello
execute if score %r0 _r matches 2.. if score %r0 _r matches ..0 run say hello
say guaranteed 1
say guaranteed 2
say guaranteed 3
say guaranteed 4
