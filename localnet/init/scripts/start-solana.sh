#!/usr/bin/env bash

set -e

cp -R localnet/init/solana /tmp

solana-test-validator --ledger /tmp/solana/test-ledger > /tmp/solana/solana.log 2>&1 &
