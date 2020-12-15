#!/bin/bash -e
echo "Running diesel setup and migrations..."
diesel setup
diesel migration run
echo "Running roach-server..."
roach-server
