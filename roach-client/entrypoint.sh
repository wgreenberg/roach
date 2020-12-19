#!/bin/bash
set -ex

while true; do
    roach-client -b cli-engine -m matchmaking -s https://roach.rodeo -t "$API_TOKEN" -- -n "$N_ITERATIONS" -d "$MAX_DEPTH"
done
