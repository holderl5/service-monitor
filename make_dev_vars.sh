#!/bin/bash

res=$(cat resources.json | jq -r '@json' | sed -e 's%\\%%g')
con=$(cat config.json | jq -r '@json' | sed -e 's%\\%%g')

echo "RESOURCES = $res" > .dev.vars
echo "CONFIG = $con" >> .dev.vars
