#!/bin/sh
cd -- "$(dirname -- "$0")" &&\
    cargo +nightly-2018-05-09 build --release --target wasm32-unknown-unknown &&\
    wasm-bindgen --no-typescript --no-modules target/wasm32-unknown-unknown/release/cyberphage_wasm.wasm --out-dir . &&\
    zip cyberphage.zip *.wasm *.js *.html rot.js/*
