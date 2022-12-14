#!/usr/bin/env bash

set -e

source "$(dirname -- "$0")/env.sh"

cargo +esp build --release
esptool.py --chip esp32 elf2image target/xtensa-esp32-espidf/release/esp32_bme280
espflash partition-table partitions.csv --to-binary > /tmp/partitions.bin

esptool --baud 921600 erase_region 0x120000 0x280000
esptool.py --baud 921600 write_flash 0x8000 /tmp/partitions.bin
esptool.py --baud 921600 write_flash 0x10000 target/xtensa-esp32-espidf/release/esp32_bme280.bin

exec espmonitor /dev/ttyUSB0
