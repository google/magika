#!/bin/bash

# A simple shell script that demonstrates basic bash features

GREETING="Hello, World!"

echo "$GREETING"

for i in 1 2 3 4 5; do
    if [ $((i % 2)) -eq 0 ]; then
        echo "$i is even"
    else
        echo "$i is odd"
    fi
done

count_files() {
    local dir="$1"
    echo "Files in $dir: $(ls "$dir" | wc -l)"
}

count_files /tmp
