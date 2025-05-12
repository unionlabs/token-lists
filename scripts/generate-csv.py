import os
import json
import csv

DATA_DIR = "data"
CSV_FILE = "tokens.csv"

fieldnames = ["symbol", "address", "decimals", "chainId", "dune_contract_address", "dune_blockchain"]

with open(CSV_FILE, "w", newline="") as csvfile:
    writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
    writer.writeheader()

    for subdir in os.listdir(DATA_DIR):
        subdir_path = os.path.join(DATA_DIR, subdir)
        tokenlist_path = os.path.join(subdir_path, "tokenlist.json")

        if os.path.isdir(subdir_path) and os.path.isfile(tokenlist_path):
            with open(tokenlist_path, "r") as f:
                try:
                    data = json.load(f)
                    tokens = data.get("tokens", [])
                    for token in tokens:
                        dune_data = token.get("extensions", {}).get("dune", {})
                        writer.writerow({
                            "symbol": token.get("symbol", ""),
                            "address": token.get("address", ""),
                            "decimals": token.get("decimals", ""),
                            "chainId": token.get("chainId", ""),
                            "dune_contract_address": dune_data.get("contract_address") or "none",
                            "dune_blockchain": dune_data.get("blockchain", "")
                        })
                except Exception as e:
                    print(f"Error reading {tokenlist_path}: {e}")
