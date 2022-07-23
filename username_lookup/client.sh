#!/usr/bin/env bash

cd $(pwd)/username_lookup
source bin/activate
python3 client.py "$@"
