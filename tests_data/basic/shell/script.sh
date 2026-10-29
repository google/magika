#!/bin/bash

# A simple shell script that greets the user

NAME=${1:-"World"}

greet() {
    echo "Hello, $NAME!"
}

if [ -z "$NAME" ]; then
    echo "Usage: $0 <name>"
    exit 1
fi

greet
exit 0
