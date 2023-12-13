scoreboard objectives add int dummy
scoreboard players set %rtest_main0 _r 4
scoreboard players set %rtest_main1 _r 4
scoreboard players set %rtest_main2 _r 4
scoreboard players set %rtest_main3 _r 4
scoreboard players set output int 4
scoreboard players remove output int 1
scoreboard players operation output int /= %l119 _l
scoreboard players add output int 1
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
scoreboard players remove output int 13924
scoreboard players operation output int /= %l4214 _l
scoreboard players add output int 118
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
