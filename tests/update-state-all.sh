#!/bin/bash
[[ $# -ne 1 ]] && echo "Provide Status" && exit 1

curl -X POST -d "{\"profile_status\": \"${1}\", \"download_size_mb\": 0}" 127.0.0.1:8000/users
