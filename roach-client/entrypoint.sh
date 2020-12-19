#!/bin/bash
set -ex

while true; do
    roach-client -b cli-engine -m matchmaking -s https://roach.rodeo -t "$API_TOKEN" -- -n "$N_ITERATIONS" -d "$MAX_DEPTH"
    if [ $? -ne 0 ]; then
        echo "roach-client exited w/ error status, exiting"
        exit
    fi
done
