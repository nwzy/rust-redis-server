#!/bin/bash

# Usage: ./script.sh <number_of_connections>
# Default to 1 connection if no argument is provided
NUM_CONNS=${1:-1}

for ((i=1; i<=NUM_CONNS; i++))
do
    # Start nc in the background to maintain the connection
    # Redirecting input from /dev/zero or a pipe keeps the connection open
    nc -v localhost 6379 < /dev/zero > /dev/null 2>&1 &

    echo "Started connection $i (PID: $!)"
done

echo "Successfully initiated $NUM_CONNS connections."
wait