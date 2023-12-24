# === dpc:init ===
scoreboard objectives add _r dummy

# === test:main ===
execute as @s at @a[tag=foo] if score %rtest_main0 _r = @s bar if score foo bar < @r[gamemode="Bar"] risk store result score %rtest_main1 _r run data get entity @s name
