#!/bin/bash

cargo build

sudo mv target/debug/thorlang /usr/bin/
