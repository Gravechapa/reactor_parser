#!/bin/bash
cargo build
g++ main.cpp -L ../target/debug/ -lreactor_parser  -Wl,-rpath=../target/debug/ -g
