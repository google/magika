#!/bin/bash

is_prime() {
    local n=$1
    if [ "$n" -lt 2 ]; then
        return 1
    fi
    for ((i = 2; i * i <= n; i++)); do
        if [ $((n % i)) -eq 0 ]; then
            return 1
        fi
    done
    return 0
}

for i in $(seq 2 50); do
    if is_prime "$i"; then
        printf "%d " "$i"
    fi
done
echo
