#!/bin/sh

echo -e "Attempting to publish project, please ensure the following: \n" \
    "- All changes have been pushed to git \n" \
    "- Semver has been updated in Cargo.toml \n" \
    "- You are sucessfully loged into crates.io via 'cargo login'\n" \

read -p "Do you wish to proceed? [y/n]" -n 1 -r
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    exit 1
fi


cargo test
cargo package
cargo publish