#!/usr/bin/env bash

./init.sh

rm -f dev_reply.log
sleep 5
while [ ! -e dev_reply.log ]; do
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c state.yaml
    sleep 10
    echo "waiting for the dev"
done

if cat /dev/shm/mode.yaml | awk -F : '{print $2}' | grep 3 >/dev/null; then
    echo "Alread in corr mode"
elif cat /dev/shm/mode.yaml | awk -F : '{print $2}' | grep 4 >/dev/null; then
    echo "BB mode, stopping"
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c stop.yaml
    rm -f dev_reply.log
    sleep 5
    while [ ! -e dev_reply.log ]; do
        cargo run --bin send --release -- --add 192.168.1.88:8888 -c state.yaml
        sleep 10
        echo "waiting for the dev"
    done

fi
cargo run --bin send --release -- --add 192.168.1.88:8888 -c start.yaml

rm -f dev_reply.log

cargo run --bin send --release -- --add 192.168.1.88:8888 -c state.yaml

while [ ! -e dev_reply.log ]; do
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c state.yaml
    sleep 10
    echo "waiting for the dev"
done

echo "started"

while :; do
    sleep 10
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c state.yaml
    date
done
