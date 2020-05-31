#!/bin/bash
sudo dtrace -c './target/release/rokumon' -o out.stacks -n 'profile-997 /execname == "rokumon"/ { @[ustack(100)] = count(); }'
