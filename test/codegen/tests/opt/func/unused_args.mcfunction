# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %atest_uses_all0 _r 0
scoreboard players set %atest_uses_all1 _r 1
scoreboard players set %atest_uses_all2 _r 2
scoreboard players set %atest_uses_all3 _r 3
scoreboard players set %atest_uses_all4 _r 4
function test:uses_all
scoreboard players set %atest_uses_one0 _r 1
function test:uses_one
scoreboard players set %atest_uses_two0 _r 1
scoreboard players set %atest_uses_two1 _r 4
function test:uses_two

# === test:uses_all === #

# === test:uses_one === #

# === test:uses_two === #
