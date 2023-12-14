scoreboard players set %rtest_main0 _r 18
scoreboard players set %rtest_main0 _r 104
scoreboard players set %rtest_main1 _r -400
scoreboard players operation %rtest_main0 _r %= %l3600 _l
execute if score %rtest_main0 _r matches 1800.. run scoreboard players set %rtest_main1 _r 400
execute store result score %rtest_main2 _r run scoreboard players operation %rtest_main0 _r %= %l1800 _l
scoreboard players operation %rtest_main2 _r *= %rtest_main0 _r
scoreboard players operation %rtest_main0 _r *= %rtest_main1 _r
scoreboard players add %rtest_main2 _r 4050000
scoreboard players operation %rtest_main0 _r /= %rtest_main2 _r
execute if score %rtest_main1 _r matches 400 run scoreboard players add %rtest_main0 _r 1
