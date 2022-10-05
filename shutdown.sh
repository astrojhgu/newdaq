#!/usr/bin/env bash

echo $1
if [ x$1 == xa2b1c4d5 ]
then
    echo Shuting down daq device in 10 secs
    sleep 10
    cargo run --bin send --release -- --add 192.168.1.88:8888 -c shutdown.yaml
    echo "echo Shutdown complete"
else
    echo "Password not correct"
    echo "exit"
fi
