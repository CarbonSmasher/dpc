# === dpc:init ===
scoreboard objectives add _r dummy

# === dpc:s/ ===
say I should be the shortest ID since I am called the most
scoreboard players set %rdpc_s_0 _r 7

# === dpc:s/b ===
scoreboard players set %rdpc_s_b0 _r 10
scoreboard players set %rdpc_s_b0 _r 11
say 1

# === dpc:s/c ===
scoreboard players set %rdpc_s_c0 _r 10
scoreboard players set %rdpc_s_c0 _r 11
say 2

# === dpc:s/d ===
scoreboard players set %rdpc_s_d0 _r 10
scoreboard players set %rdpc_s_d0 _r 11
say 3

# === sh:ort ===
say I should not be stripped because my ID is already shorter than the stripped version
scoreboard players set %rsh_ort0 _r 7

# === test:dont_strip_me ===
say Don't strip me, I am marked with preserve

# === test:main ===
function sh:ort
function test:dont_strip_me
function dpc:s/
function dpc:s/
function dpc:s/
function dpc:s/b
function dpc:s/c
function dpc:s/d
