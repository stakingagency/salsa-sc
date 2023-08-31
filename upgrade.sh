#!/bin/bash
mxpy contract build
mxpy --verbose contract upgrade erd1qqqqqqqqqqqqqpgqpk3qzj86tme9kzxdq87f2rdf5nlwsgvjvcqs5hke3x --bytecode output/salsa.wasm --recall-nonce --pem=~/walletKey.pem --send --proxy="https://devnet-gateway.multiversx.com" --chain="D" --metadata-payable-by-sc --metadata-payable --gas-limit=300000000 --outfile="salsa.json"
rm salsa.json
