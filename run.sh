#!/usr/bin/env bash

cp -f cfg.yaml /dev/shm/

tmux ls |grep daq && tmux kill-session -t daq
tmux new -s daq -d
tmux split-window -t daq -d
tmux split-window -t daq -d
tmux split-window -t daq -d

for i in $(seq 4); do
    tmux next-layout -t daq
    tmux next-layout -t daq
    tmux next-layout -t daq
    tmux next-layout -t daq
    tmux next-layout -t daq
done

tmux send-keys -t daq.2 "cargo run --bin server --release -- -a 0.0.0.0:8888" C-m
tmux send-keys -t daq.3 "./start.sh" C-m
tmux send-keys -t daq.0 "./daq.sh" C-m
tmux send-keys -t daq.1 "./diskmgr.sh" C-m


#tmux attach -t daq.1

######

tmux ls |grep monitor && tmux kill-session -t monitor
tmux new -s monitor -d
tmux split-window -t monitor -d
tmux next-layout -t monitor
tmux send-keys -t monitor.1 "cargo run --bin monitor --release" C-m
tmux send-keys -t monitor.0 "cd monitor_server; python -m http.server --bind ::" C-m
