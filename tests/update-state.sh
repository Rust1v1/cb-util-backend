#!/bin/bash
[[ $# -ne 2 ]] && echo "Provide Username and Status" && exit 1

curl -X POST -d "{\"profile_status\": \"${2}\", \"download_size_mb\": 0}" 127.0.0.1:8000/users/${1}
