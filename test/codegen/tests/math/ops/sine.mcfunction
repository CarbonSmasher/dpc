# === dpc:init === #
scoreboard objectives add _r dummy
scoreboard objectives add _l dummy
scoreboard players set %l1800 _l 1800
scoreboard players set %l3600 _l 3600

# === test:main === #
scoreboard players set %rtest_main0 _r 18
scoreboard players set %rtest_main0 _r 18
scoreboard players set %rtest_main0 _r 24

# === test:sine === #
scoreboard players operation %rtest_sine0 _r = %atest_sine0 _r
scoreboard players set %rtest_sine1 _r -400
scoreboard players operation %rtest_sine0 _r %= %l3600 _l
execute if score %rtest_sine0 _r matches 1800.. run scoreboard players set %rtest_sine1 _r 400
execute store result score %rtest_sine2 _r run scoreboard players operation %rtest_sine0 _r %= %l1800 _l
scoreboard players remove %rtest_sine2 _r 1800
scoreboard players operation %rtest_sine2 _r *= %rtest_sine0 _r
scoreboard players operation %rtest_sine0 _r *= %rtest_sine1 _r
scoreboard players add %rtest_sine2 _r 4050000
scoreboard players operation %rtest_sine0 _r /= %rtest_sine2 _r
execute if score %rtest_sine1 _r matches 400 run scoreboard players add %rtest_sine0 _r 1
scoreboard players operation %Rtest_sine0 _r = %rtest_sine0 _r
