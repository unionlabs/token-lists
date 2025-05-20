use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::sol;
use alloy::transports::http::reqwest::Url;
use alloy::transports::http::{Client, Http};
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use std::fs;
use std::str::FromStr;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function name() view returns (string);
        function symbol() view returns (string);
        function decimals() view returns (uint8);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Token {
    address: String,
    symbol: String,
    name: String,
    decimals: u8,
}

#[derive(Deserialize)]
struct TokenInfoResponse {
    data: TokenInfo,
}

#[derive(Deserialize)]
struct TokenInfo {
    name: String,
    symbol: String,
    decimals: u8,
}

#[tokio::main] // <- ADICIONA ISTO
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        ("ethereum", "../../data/ethereum.1/tokenlist.json"),
        ("bob", "../../data/bob.60808/tokenlist.json"),
        ("corn", "../../data/corn.21000000/tokenlist.json"),
        ("babylon", "../../data/babylon.bbn-1/tokenlist.json"),
    ];

    for (network, path) in files {
        println!("\n🔍 Checking file: {path} for network: {network}");

        let content = fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        let token_array = json.get("tokens").ok_or("Missing 'tokens' field")?;
        let tokens: Vec<Token> = serde_json::from_value(token_array.clone())?;

        if let Some(provider) = get_ethereum_provider(network) {
            for token in tokens {
                verify_on_ethereum(&token, &provider).await?;
            }
        } else if network == "babylon" {
            for token in tokens {
                verify_on_babylon(&token).await?;
            }
        } else {
            println!("⚠️ Unknown network: {}", network);
        }
    }

    Ok(())
}

async fn verify_on_ethereum(
    token: &Token,
    provider: &RootProvider<Http<Client>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if token.address.to_lowercase() == "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" {
        println!("ℹ️ Skipping native placeholder token: {}", token.symbol);
        return Ok(());

    }

    let address = Address::from_str(&token.address)?;
    let contract = IERC20::new(address, &provider);

    let on_chain_symbol = contract.symbol().call().await?._0;
    let on_chain_name = contract.name().call().await?._0;
    let on_chain_decimals = contract.decimals().call().await?._0;

    let mut is_valid = true;

    if token.symbol != on_chain_symbol {
        println!(
            "[{}] ❌ Symbol mismatch: JSON = {}, On-chain = {} ",
            token.symbol, token.symbol, on_chain_symbol
        );
        is_valid = false;
    }

    if token.name != on_chain_name {
        println!(
            "[{}] ❌ Name mismatch: JSON = {}, On-chain = {} ",
            token.symbol, token.name, on_chain_name
        );
        is_valid = false;
    }

    if token.decimals != on_chain_decimals {
        println!(
            "[{}] ❌ Decimals mismatch: JSON = {}, On-chain = {} ",
            token.symbol, token.decimals, on_chain_decimals
        );
        is_valid = false;
    }

    if is_valid {
        println!("[{}] ✅ Token is valid ", token.symbol);
    }

    Ok(())
}

fn get_ethereum_provider(network: &str) -> Option<RootProvider<Http<Client>>> {
    let url = match network {
        "ethereum" => "x",
        "bob" => "x",
        "corn" => "x",
        _ => return None,
    };

    let provider = ProviderBuilder::new().on_http(Url::parse(url).ok()?);

    Some(provider)
}

async fn verify_on_babylon(token: &Token) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let msg = serde_json::json!({ "token_info": {} });
    let query = general_purpose::STANDARD.encode(msg.to_string());

    let rpc_url = "https://babylon.nodes.guru";
    let url = format!(
        "{}/api/cosmwasm/wasm/v1/contract/{}/smart/{}",
        rpc_url, token.address, query
    );

    if token.address == "ubbn" {
        println!("ℹ️ Skipping non-contract token: {}", token.symbol);
        return Ok(());
    }

    if token.address.starts_with("ibc/") {
        println!("ℹ️ Skipping IBC token: {}", token.symbol);
        return Ok(());
    }

    let res: TokenInfoResponse = client.get(&url).send().await?.json().await?;

    let mut is_valid = true;

    if token.symbol != res.data.symbol {
        println!(
            "[{}] ❌ Symbol mismatch: JSON = {}, On-chain = {}",
            token.symbol, token.symbol, res.data.symbol
        );
        is_valid = false;
    }

    if token.name != res.data.name {
        println!(
            "[{}] ❌ Name mismatch: JSON = {}, On-chain = {}",
            token.symbol, token.name, res.data.name
        );
        is_valid = false;
    }

    if token.decimals != res.data.decimals {
        println!(
            "[{}] ❌ Decimals mismatch: JSON = {}, On-chain = {}",
            token.symbol, token.decimals, res.data.decimals
        );
        is_valid = false;
    }

    if is_valid {
        println!("[{}] ✅ Token is valid ", token.symbol);
    }

    Ok(())
}
