# === dpc:init ===
data merge storage dpc:r {}

# === test:main ===
data modify storage dpc:r stest_main0 set value [7,3]
data modify storage dpc:r stest_main0 append value 10
