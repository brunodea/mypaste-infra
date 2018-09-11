#!/bin/bash

curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"content":"xyz"}' \
  http://localhost:8088/paste
