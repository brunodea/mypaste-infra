#!/bin/bash

curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"content":"xyz"}' \
  http://127.0.0.1:8088/paste
