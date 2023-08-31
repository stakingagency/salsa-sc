#!/bin/bash
mxpy --verbose contract upgrade erd1qqqqqqqqqqqqqpgqpk3qzj86tme9kzxdq87f2rdf5nlwsgvjvcqs5hke3x --bytecode output-docker/salsa/salsa.wasm --recall-nonce --pem=~/walletKey.pem --send --proxy="https://devnet-gateway.multiversx.com" --chain="D" --outfile="deploy-devnet.interaction.json" --metadata-payable-by-sc --gas-limit=300000000

