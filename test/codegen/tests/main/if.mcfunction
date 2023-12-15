scoreboard players set %rtest_main0 _r 7
execute if score %rtest_main0 _r matches 7 run say hello
execute if score %rtest_main1 _r matches ..2147483647 unless score %l5 _l = %rtest_main1 _r run scoreboard players set %rtest_main0 _r 3
