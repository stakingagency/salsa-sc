#!/bin/bash
mxpy --verbose contract deploy --bytecode output-docker/salsa/salsa.wasm --recall-nonce --pem=~/walletKey.pem --send --proxy="https://devnet-gateway.multiversx.com" --chain="D" --outfile="deploy-devnet.interaction.json" --metadata-payable-by-sc --gas-limit=100000000

