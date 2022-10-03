#!/bin/bash
cargo build --release || exit
cat cfg.yaml >/dev/shm/cfg.yaml

getcap ./target/release/save_data |grep cap_net_admin,cap_net_raw=eip || sudo setcap cap_net_raw,cap_net_admin=eip ./target/release/save_data
dev1=enp101s0f0

taskset --cpu-list 8-15 numactl -N netdev:$dev1 --localalloc ./target/release/save_data -c cfg.yaml



