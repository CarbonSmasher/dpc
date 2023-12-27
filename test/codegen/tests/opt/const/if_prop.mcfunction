# === dpc:init ===
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l1 _l 1

# === test:main ===
execute if score foo bar matches 5.. run say hello
scoreboard players set %rtest_main0 _r 10
scoreboard players operation %rtest_main1 _r = @s foo
execute if score %rtest_main1 _r matches 15 run scoreboard players add %rtest_main0 _r 15
scoreboard players operation %rtest_main1 _r = @s bar
execute if score %rtest_main1 _r matches 1 run scoreboard players operation %rtest_main0 _r *= %l1 _l
scoreboard players operation %rtest_main0 _r += %rtest_main1 _r
