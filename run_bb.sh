#!/usr/bin/env bash

cp -f cfg.yaml /dev/shm/

tmux ls |grep daq && tmux kill-session -t daq
tmux new -s daq -d
tmux split-window -t daq -d
tmux next-layout -t daq


tmux send-keys -t daq.0 "cargo run --bin server --release -- -a 0.0.0.0:8888" C-m
tmux send-keys -t daq.1 "./start_bb.sh" C-m

