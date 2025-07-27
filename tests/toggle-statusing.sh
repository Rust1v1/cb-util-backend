#!/bin/bash

ENABLE=$1
[[ -z $ENABLE ]] && ENABLE=false
curl -X POST -d "{\"statusing\": ${ENABLE}}" http://127.0.0.1:8000/statusing
