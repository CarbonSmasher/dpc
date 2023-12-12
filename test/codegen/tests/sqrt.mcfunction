scoreboard objectives add int dummy
scoreboard players set %rtest_main0 _r 4
scoreboard players set %rtest_main1 _r 4
scoreboard players set %rtest_main2 _r 4
scoreboard players set %rtest_main3 _r 4
scoreboard players set output int 4
function test:sqrt1
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
scoreboard players set %rtest_main0 _r 1234997
scoreboard players set %rtest_main1 _r 1234997
scoreboard players set %rtest_main2 _r 1234997
scoreboard players set %rtest_main3 _r 1234997
scoreboard players set output int 1234997
function test:sqrt2
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
