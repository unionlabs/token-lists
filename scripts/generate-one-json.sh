#!/usr/bin/env bash

set -eou pipefail

output="{"

for dir in data/*/; do
  token_list_dir=$(basename "$dir")
  json_file="$dir/tokenlist.json"
  
  if [ -f "$json_file" ]; then
    json_content=$(jq -c . "$json_file")
    output+="\"$token_list_dir\": $json_content,"
  fi
done

# Remove the trailing comma and close the JSON object
output="${output%,}}"

echo "$output" | jq .
