######## main ########
# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l2 _l 2
scoreboard players set %l119 _l 119
scoreboard players set %l4214 _l 4214
scoreboard players set %l50436 _l 50436

# === test:main === #
scoreboard players set %atest_sqrt.0 _r 4
function test:sqrt
scoreboard players operation %rtest_main.0 _r = %Rtest_sqrt.0 _r
scoreboard players set %atest_sqrt.0 _r 25
function test:sqrt
scoreboard players operation %rtest_main.0 _r = %Rtest_sqrt.0 _r

# === test:sqrt === #
scoreboard players operation %rtest_sqrt.0 _r = %atest_sqrt.0 _r
scoreboard players operation %rtest_sqrt.1 _r = %atest_sqrt.0 _r
scoreboard players operation %rtest_sqrt.2 _r = %atest_sqrt.0 _r
scoreboard players operation %rtest_sqrt.3 _r = %atest_sqrt.0 _r
scoreboard players operation %rtest_sqrt.4 _r = %atest_sqrt.0 _r
execute if score %atest_sqrt.0 _r matches ..13924 run function test:sqrt_body_0
execute if score %atest_sqrt.0 _r matches 13925.. if score %atest_sqrt.0 _r matches ..16777216 run function test:sqrt_body_1
execute if score %atest_sqrt.0 _r matches 16777217.. run function test:sqrt_body_2
scoreboard players operation %rtest_sqrt.0 _r /= %rtest_sqrt.4 _r
scoreboard players operation %rtest_sqrt.4 _r += %rtest_sqrt.0 _r
scoreboard players operation %rtest_sqrt.4 _r /= %l2 _l
scoreboard players operation %rtest_sqrt.1 _r /= %rtest_sqrt.4 _r
scoreboard players operation %rtest_sqrt.4 _r += %rtest_sqrt.1 _r
scoreboard players operation %rtest_sqrt.4 _r /= %l2 _l
scoreboard players operation %rtest_sqrt.2 _r /= %rtest_sqrt.4 _r
scoreboard players operation %rtest_sqrt.4 _r += %rtest_sqrt.2 _r
scoreboard players operation %rtest_sqrt.4 _r /= %l2 _l
scoreboard players operation %rtest_sqrt.3 _r /= %rtest_sqrt.4 _r
scoreboard players operation %rtest_sqrt.4 _r += %rtest_sqrt.3 _r
scoreboard players operation %rtest_sqrt.4 _r /= %l2 _l
scoreboard players operation %Rtest_sqrt.0 _r = %rtest_sqrt.4 _r

# === test:sqrt_body_0 === #
scoreboard players remove %rtest_sqrt.4 _r 1
scoreboard players operation %rtest_sqrt.4 _r /= %l119 _l
scoreboard players add %rtest_sqrt.4 _r 1

# === test:sqrt_body_1 === #
scoreboard players remove %rtest_sqrt.4 _r 13924
scoreboard players operation %rtest_sqrt.4 _r /= %l4214 _l
scoreboard players add %rtest_sqrt.4 _r 118

# === test:sqrt_body_2 === #
scoreboard players remove %rtest_sqrt.4 _r 16777216
scoreboard players operation %rtest_sqrt.4 _r /= %l50436 _l
scoreboard players add %rtest_sqrt.4 _r 4096

######## opt ########
# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main.0 _r 2
scoreboard players set %rtest_main.0 _r 5
