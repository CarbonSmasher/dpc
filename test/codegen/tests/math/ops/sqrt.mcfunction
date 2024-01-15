######## main ########
# === dpc::ifbody_0 === #
scoreboard players remove %rtest_sqrt4 _r 1
scoreboard players operation %rtest_sqrt4 _r /= %l119 _l
scoreboard players add %rtest_sqrt4 _r 1

# === dpc::ifbody_1 === #
scoreboard players remove %rtest_sqrt4 _r 13924
scoreboard players operation %rtest_sqrt4 _r /= %l4214 _l
scoreboard players add %rtest_sqrt4 _r 118

# === dpc::ifbody_2 === #
scoreboard players remove %rtest_sqrt0 _r 16777216
scoreboard players operation %rtest_sqrt0 _r /= %l50436 _l
scoreboard players add %rtest_sqrt0 _r 4096

# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l2 _l 2
scoreboard players set %l119 _l 119
scoreboard players set %l4214 _l 4214
scoreboard players set %l50436 _l 50436

# === test:main === #
scoreboard players set atest_sqrt0 _r 4
function test:sqrt
scoreboard players operation %rtest_main4 _r = rtest_sqrt0 _r
scoreboard players set atest_sqrt0 _r 25
function test:sqrt
scoreboard players operation %rtest_main4 _r = rtest_sqrt0 _r

# === test:sqrt === #
scoreboard players operation %rtest_sqrt0 _r = %atest_sqrt0 _r
scoreboard players operation %rtest_sqrt1 _r = %atest_sqrt0 _r
scoreboard players operation %rtest_sqrt2 _r = %atest_sqrt0 _r
scoreboard players operation %rtest_sqrt3 _r = %atest_sqrt0 _r
scoreboard players operation %rtest_sqrt4 _r = %atest_sqrt0 _r
execute if score %atest_sqrt0 _r matches ..13924 run function dpc::ifbody_0
execute if score %atest_sqrt0 _r matches ..16777216 if score %atest_sqrt0 _r matches 13925.. run function dpc::ifbody_1
execute if score %atest_sqrt0 _r matches 16777217.. run function dpc::ifbody_2
scoreboard players operation %rtest_sqrt0 _r /= %rtest_sqrt4 _r
scoreboard players operation %rtest_sqrt4 _r += %rtest_sqrt0 _r
scoreboard players operation %rtest_sqrt4 _r /= %l2 _l
scoreboard players operation %rtest_sqrt1 _r /= %rtest_sqrt4 _r
scoreboard players operation %rtest_sqrt4 _r += %rtest_sqrt1 _r
scoreboard players operation %rtest_sqrt4 _r /= %l2 _l
scoreboard players operation %rtest_sqrt2 _r /= %rtest_sqrt4 _r
scoreboard players operation %rtest_sqrt4 _r += %rtest_sqrt2 _r
scoreboard players operation %rtest_sqrt4 _r /= %l2 _l
scoreboard players operation %rtest_sqrt3 _r /= %rtest_sqrt4 _r
scoreboard players operation %rtest_sqrt4 _r += %rtest_sqrt3 _r
scoreboard players operation %rtest_sqrt4 _r /= %l2 _l
scoreboard players operation %Rtest_sqrt0 _r = %rtest_sqrt4 _r

######## opt ########
# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main0 _r 2
scoreboard players set %rtest_main0 _r 5
