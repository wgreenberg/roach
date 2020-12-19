#!/bin/bash
set -x

roach-client -b cli-engine -a "-n \"$N_ITERATIONS\" -d \"$MAX_DEPTH\"" matchmaking -s https://roach.rodeo -t "$API_TOKEN"
