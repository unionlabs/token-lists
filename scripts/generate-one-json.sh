#!/usr/bin/env bash

set -eou pipefail

output="{"

# Flag to check if we added any tokens
first=true

for dir in data/*/; do
  token_list_dir=$(basename "$dir")
  json_file="$dir/tokenlist.json"
  
  if [ -f "$json_file" ]; then
    json_content=$(jq -c . "$json_file")
    
    if [ "$first" = true ]; then
      first=false
    else
      output+=","
    fi
    
    output+="\"$token_list_dir\": $json_content"
  fi
done

# Close the JSON object
output+="}"

echo "$output" | jq .
