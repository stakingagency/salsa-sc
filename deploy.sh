#!/bin/bash
mxpy contract build
mxpy --verbose contract deploy --bytecode output/salsa.wasm --recall-nonce --pem=~/walletKey.pem --send --proxy="https://devnet-gateway.multiversx.com" --chain="D" --metadata-payable-by-sc --metadata-payable --gas-limit=200000000 --outfile="salsa.json"
rm salsa.json
