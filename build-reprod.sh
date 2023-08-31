#!/bin/bash
sudo rm -rf output-docker
sudo /home/mihai/multiversx-sdk/mxpy contract reproducible-build --docker-image="multiversx/sdk-rust-contract-builder:v5.0.0"
