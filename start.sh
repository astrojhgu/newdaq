#!/usr/bin/env bash

rm -f dev_reply.log
sleep 5
while [ ! -e dev_reply.log ]; do
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c status.yaml
    sleep 10
    echo "waiting for the dev"
done
cargo run --bin send --release -- --add 192.168.1.88:8888 -c start.yaml
sleep 10

rm -f dev_reply.log
cargo run --bin send --release -- --add 192.168.1.88:8888 -c status.yaml
while [ ! -e dev_reply.log ]; do
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c status.yaml
    sleep 10
    echo "waiting for the dev"
done

echo "started"

while :; do
    sleep 10
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c status.yaml
    date
done
