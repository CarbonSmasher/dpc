# === dpc:init === #
scoreboard objectives add _r dummy

# === test:main === #
scoreboard players set %rtest_main0 _r -7
scoreboard players set %rtest_main0 _r 1
scoreboard players set %rtest_main0 _r 0
data modify storage dpc:r rtest_main0 set value 20b
data modify storage dpc:r rtest_main0 set value 1b
data modify storage dpc:r rtest_main0 set value 0b
data modify storage dpc:r rtest_main0 set value 6s
data modify storage dpc:r rtest_main0 set value 6
data modify storage dpc:r rtest_main0 set value 3242389l
data modify storage dpc:r rtest_main0 set value "hello world"
data modify storage dpc:r rtest_main0 set value []
data modify storage dpc:r rtest_main0 append value "foo"
data modify storage dpc:r rtest_main0 set value ["foo","bar","baz"]
data modify storage dpc:r rtest_main0 set value [B;0b,0b,0b,0b,0b,0b,0b,0b,0b,0b]
data modify storage dpc:r rtest_main0 prepend value 0b
data modify storage dpc:r rtest_main0 set value [B;7b,3b,-4b,8b,3b,1b,0b,7b,-12b,0b]
data modify storage dpc:r rtest_main0 set value 5f
data modify storage dpc:r rtest_main0 set value -.20045
data modify storage dpc:r rtest_main0 set value [I;7,8,3]
data modify storage dpc:r rtest_main0 set value {bar:[4,6],baz:{foo:0b},foo:"bar"}
