#!/bin/bash

curl --header "Content-Type: application/json" \
  --request POST \
  --data '{"PlainText":"MyTest"}' \
  http://localhost:8000/paste
