######## main ########
# === dpc::ifbody_0 === #
scoreboard players operation atest_sqrt10 _r = %rdpc__ifbody_04 _r
function test:sqrt1
scoreboard players operation %rdpc__ifbody_04 _r = rtest_sqrt10 _r

# === dpc::ifbody_1 === #
scoreboard players operation atest_sqrt20 _r = %rdpc__ifbody_10 _r
function test:sqrt2
scoreboard players operation %rdpc__ifbody_10 _r = rtest_sqrt20 _r

# === dpc::ifbody_2 === #
scoreboard players operation atest_sqrt30 _r = %rdpc__ifbody_20 _r
function test:sqrt3
scoreboard players operation %rdpc__ifbody_20 _r = rtest_sqrt30 _r

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
execute if score %atest_sqrt0 _r matches 13925.. if score %atest_sqrt0 _r matches ..16777216 run function dpc::ifbody_1
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

# === test:sqrt1 === #
scoreboard players remove %atest_sqrt10 _r 1
scoreboard players operation %atest_sqrt10 _r /= %l119 _l
scoreboard players add %atest_sqrt10 _r 1
scoreboard players operation %Rtest_sqrt10 _r = %atest_sqrt10 _r

# === test:sqrt2 === #
scoreboard players remove %atest_sqrt20 _r 13924
scoreboard players operation %atest_sqrt20 _r /= %l4214 _l
scoreboard players add %atest_sqrt20 _r 118
scoreboard players operation %Rtest_sqrt20 _r = %atest_sqrt20 _r

# === test:sqrt3 === #
scoreboard players remove %atest_sqrt30 _r 16777216
scoreboard players operation %atest_sqrt30 _r /= %l50436 _l
scoreboard players add %atest_sqrt30 _r 4096
scoreboard players operation %Rtest_sqrt30 _r = %atest_sqrt30 _r

######## opt ########
# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main0 _r 2
scoreboard players set %rtest_main0 _r 5
