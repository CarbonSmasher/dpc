scoreboard players set %rtest_main0 _r 4
scoreboard players set %rtest_main1 _r 4
scoreboard players set %rtest_main2 _r 1
scoreboard players add %rtest_main2 _r 4
scoreboard players operation %rtest_main2 _r /= %l2 _l
scoreboard players operation %rtest_main0 _r /= %rtest_main2 _r
scoreboard players operation %rtest_main2 _r += %rtest_main0 _r
scoreboard players operation %rtest_main2 _r /= %l2 _l
scoreboard players operation %rtest_main1 _r /= %rtest_main2 _r
scoreboard players operation %rtest_main2 _r += %rtest_main1 _r
execute store result score %rtest_main1 _r run scoreboard players operation %rtest_main2 _r /= %l2 _l
