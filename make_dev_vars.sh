#!/bin/bash

res=$(cat resources.json | jq -r '@json' | sed -e 's%\\%%g')
con=$(cat config.json | jq -r '@json' | sed -e 's%\\%%g')

echo "RESOURCES = $res" > .dev.vars
echo "CONFIG = $con" >> .dev.vars

# echo the data for wrangler.toml for now:
echo "Replace vars in wrangler.toml with the following:"
echo
echo "[vars]"
echo "RESOURCES = $(cat resources.json | jq '@json')"
echo "CONFIG=$(cat config.json | jq '@json')"
