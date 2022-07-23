#!/usr/bin/env bash

cd $(pwd)/username_lookup
virtualenv .
source bin/activate
$(pwd)/bin/python3 -m pip install --upgrade pip
pip install -r requirements.txt
