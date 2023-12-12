scoreboard objectives add int dummy
scoreboard players set input int 4
scoreboard players operation %rtest_main0 _r = input int
scoreboard players operation %rtest_main1 _r = input int
scoreboard players operation %rtest_main2 _r = input int
scoreboard players operation %rtest_main3 _r = input int
scoreboard players operation output int = input int
execute if score input int matches ..13924 run function test:sqrt1
execute if score input int matches 13925..16777216 run function test:sqrt2
execute if score input int matches 16777217.. run function test:sqrt3
scoreboard players operation %rtest_main0 _r /= output int
scoreboard players operation output int += %rtest_main0 _r
scoreboard players operation output int /= %l2 _l
scoreboard players operation %rtest_main1 _r /= output int
scoreboard players operation output int += %rtest_main1 _r
scoreboard players operation output int /= %l2 _l
scoreboard players operation %rtest_main2 _r /= output int
scoreboard players operation output int += %rtest_main2 _r
scoreboard players operation output int /= %l2 _l
scoreboard players operation %rtest_main3 _r /= output int
scoreboard players operation output int += %rtest_main3 _r
scoreboard players operation output int /= %l2 _l
