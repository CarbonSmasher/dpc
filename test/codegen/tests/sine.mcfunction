scoreboard players set .in math 100
scoreboard players operation %rtest_main0 _r = .in math
scoreboard players set %rtest_main1 _r -400
scoreboard players operation %rtest_main0 _r %= %l3600 _l
execute if score %rtest_main0 _r matches 1800.. run scoreboard players set %rtest_main1 _r 400
execute store result score %rtest_main2 _r run scoreboard players operation %rtest_main0 _r %= %l1800 _l
scoreboard players remove %rtest_main2 _r 1800
scoreboard players operation %rtest_main2 _r *= %rtest_main0 _r
scoreboard players operation %rtest_main3 _r *= %rtest_main1 _r
scoreboard players add %rtest_main2 _r 4050000
scoreboard players operation %rtest_main3 _r /= %rtest_main2 _r
execute if score %rtest_main1 _r matches 400 run scoreboard players add %rtest_main3 _r 1
scoreboard players operation .out math = %rtest_main3 _r
scoreboard players set .in math 1
scoreboard players operation %rtest_main0 _r = .in math
scoreboard players set %rtest_main1 _r -400
scoreboard players operation %rtest_main0 _r %= %l3600 _l
execute if score %rtest_main0 _r matches 1800.. run scoreboard players set %rtest_main1 _r 400
execute store result score %rtest_main2 _r run scoreboard players operation %rtest_main0 _r %= %l1800 _l
scoreboard players remove %rtest_main2 _r 1800
scoreboard players operation %rtest_main2 _r *= %rtest_main0 _r
scoreboard players operation %rtest_main3 _r *= %rtest_main1 _r
scoreboard players add %rtest_main2 _r 4050000
scoreboard players operation %rtest_main3 _r /= %rtest_main2 _r
execute if score %rtest_main1 _r matches 400 run scoreboard players add %rtest_main3 _r 1
scoreboard players operation .out math = %rtest_main3 _r
