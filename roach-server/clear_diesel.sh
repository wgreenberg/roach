#!/bin/bash

dropdb roach_server
rm src/schema.rs
diesel setup && diesel migration run
