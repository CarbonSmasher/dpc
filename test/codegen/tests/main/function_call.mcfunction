scoreboard players set %rtest_main0 _r -44
scoreboard players set atest_fn0 _r 7
scoreboard players operation atest_fn1 _r = %rtest_main0 _r
data modify storage dpc:r atest_fn2 set value "foo"
function test:fn
scoreboard players operation %rtest_main0 _r = rtest_fn0 _r
