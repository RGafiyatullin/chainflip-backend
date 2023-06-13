#!/bin/bash

function a() {
    MY_ADDRESS=`pnpm tsx ./commands/new_btc_address.ts gonna P2SH`
    echo "Created new BTC address $MY_ADDRESS"
    pnpm tsx ./commands/perform_swap.ts eth btc $MY_ADDRESS
}

function b() {
    MY_ADDRESS=`pnpm tsx ./commands/new_btc_address.ts give P2WPKH`
    echo "Created new BTC address $MY_ADDRESS"
    pnpm tsx ./commands/perform_swap.ts usdc btc $MY_ADDRESS
}

function c() {
    MY_ADDRESS=`pnpm tsx ./commands/new_btc_address.ts you P2WSH` 
    echo "Created new BTC address $MY_ADDRESS"
    pnpm tsx ./commands/perform_swap.ts dot btc $MY_ADDRESS
}

function d() {
    MY_ADDRESS=`pnpm tsx ./commands/new_dot_address.ts up`
    echo "Created new DOT address $MY_ADDRESS"
    pnpm tsx ./commands/perform_swap.ts btc dot $MY_ADDRESS
}

function e() {
    MY_ADDRESS=`pnpm tsx ./commands/new_eth_address.ts and`
    echo "Created new USDC address $MY_ADDRESS"
    pnpm tsx ./commands/perform_swap.ts dot usdc $MY_ADDRESS
}

function f() {
    MY_ADDRESS=`pnpm tsx ./commands/new_eth_address.ts desert`
    echo "Created new ETH address $MY_ADDRESS"
    pnpm tsx ./commands/perform_swap.ts btc eth $MY_ADDRESS
}

function g() {
    MY_ADDRESS=`pnpm tsx ./commands/new_btc_address.ts never P2PKH`
    echo "Created new BTC address $MY_ADDRESS"
    pnpm tsx ./commands/perform_swap.ts dot btc $MY_ADDRESS
}

echo "=== Testing all swap combinations ==="

(a & b & c & d & e & f & g) & wait

echo "=== Test complete ==="