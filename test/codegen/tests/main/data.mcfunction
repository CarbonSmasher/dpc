# === test:main === #
data modify storage dpc:r rtest_main0 set value 7
data modify storage dpc:r rtest_main0 set from storage test:test foo
data modify storage dpc:r rtest_main0 set from entity @s foo
data modify storage dpc:r rtest_main0 set from block ~ ~ ~ foo
