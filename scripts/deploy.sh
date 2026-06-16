#!/usr/bin/env bash

cargo build --release && cp target/release/latuicon "$HOME/.local/bin/latuicon"
