#!/bin/bash
mxpy --verbose contract upgrade erd1qqqqqqqqqqqqqpgqaqxztq0y764dnet95jwtse5u5zkg92sfacts6h9su3 --bytecode output-docker/salsa/salsa.wasm --recall-nonce --keyfile=~/Desktop/ElrondKeys/Mainnet/erd1salsavmx35a4q30wyqsjhhqcy4dngj4jf35qmtaynzxksm4factsa6wrl0.json --send --proxy="https://gateway.multiversx.com" --chain="1" --outfile="deploy-mainnet.interaction.json" --metadata-payable-by-sc --metadata-payable --gas-limit=300000000

