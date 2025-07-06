#!/bin/bash
podman run --name mongo -d --rm -p 39329:27017 docker.io/library/mongo:latest
