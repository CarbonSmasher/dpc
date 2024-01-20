# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %atest_uses_all.0 _r 0
scoreboard players set %atest_uses_all.1 _r 1
scoreboard players set %atest_uses_all.2 _r 2
scoreboard players set %atest_uses_all.3 _r 3
scoreboard players set %atest_uses_all.4 _r 4
function test:uses_all
scoreboard players set %atest_uses_one.0 _r 1
function test:uses_one
scoreboard players set %atest_uses_two.0 _r 1
scoreboard players set %atest_uses_two.1 _r 4
function test:uses_two

# === test:uses_all === #

# === test:uses_one === #

# === test:uses_two === #
