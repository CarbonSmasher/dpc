data modify storage dpc:r s0 set from entity foo bar
scoreboard players operation %r0 _r = foo bar
data remove storage dpc:r s0
scoreboard players reset %r0 _r
data modify storage dpc:r s0 set from entity @s foo
scoreboard players operation %r0 _r = @s bar
data modify storage dpc:r s0 set from storage foo:bar bar
