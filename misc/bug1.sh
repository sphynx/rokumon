#!/bin/sh
cat game-logs/bug1.txt - | target/release/rokumon -o HumanHuman -f --cards="jjjggjg" --no-shuffle
