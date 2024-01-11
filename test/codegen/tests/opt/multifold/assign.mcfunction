# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
execute store success score %rtest_main0 _r if biome ~ ~ ~ forest
execute store success score %rtest_main0 _r unless predicate foo
