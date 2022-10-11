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
    echo "already in Correlator mode"
else
    echo "Stopping"
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c stop.yaml
    rm -f dev_reply.log
    sleep 5
    while [ ! -e dev_reply.log ]; do
        cargo run --bin send --release -- --add 192.168.1.88:8888 -c state.yaml
        sleep 10
        echo "waiting for the dev"
    done
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c start.yaml
fi


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
