use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::sol;
use alloy::transports::http::reqwest::Url;
use alloy::transports::http::{Client, Http};
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use std::env;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = vec![
        ("ethereum", "../../data/ethereum.1/tokenlist.json"),
        ("bob", "../../data/bob.60808/tokenlist.json"),
        ("corn", "../../data/corn.21000000/tokenlist.json"),
        ("babylon", "../../data/babylon.bbn-1/tokenlist.json"),
    ];

    let mut error_count = 0;

    for (network, path) in files {
        println!("\n🔍 Checking file: {path} for network: {network}");

        let content = fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        let token_array = json.get("tokens").ok_or("Missing 'tokens' field")?;
        let tokens: Vec<Token> = serde_json::from_value(token_array.clone())?;

        if let Some(provider) = get_ethereum_provider(network) {
            for token in tokens {
                match verify_on_ethereum(&token, &provider).await {
                    Ok(errors) => error_count += errors,
                    Err(e) => {
                        eprintln!("❌ Failed to verify {}: {}", token.symbol, e);
                        error_count += 1;
                    }
                }
            }
        } else if network == "babylon" {
            for token in tokens {
                match verify_on_babylon(&token).await {
                    Ok(errors) => error_count += errors,
                    Err(e) => {
                        eprintln!("❌ Failed to verify {}: {}", token.symbol, e);
                        error_count += 1;
                    }
                }
            }
        } else {
            println!("⚠️ Unknown network: {}", network);
        }
    }

    if error_count > 0 {
        eprintln!("\n Found {error_count} token validation error(s).");
        return Err(format!("Validation failed with {error_count} error(s)").into());
    }

    println!("\n✅ All tokens are valid!");
    Ok(())
}

async fn verify_on_ethereum(
    token: &Token,
    provider: &RootProvider<Http<Client>>,
) -> Result<u32, Box<dyn std::error::Error>> {
    if token.address.to_lowercase() == "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" {
        println!("ℹ️ Skipping native placeholder token: {}", token.symbol);
        return Ok(0);
    }

    let address = Address::from_str(&token.address)?;
    let contract = IERC20::new(address, &provider);

    let on_chain_symbol = contract.symbol().call().await?._0;
    let on_chain_name = contract.name().call().await?._0;
    let on_chain_decimals = contract.decimals().call().await?._0;

    let mut local_error_count = 0;

    if token.symbol != on_chain_symbol {
        println!(
            "[{}] ❌ Symbol mismatch: JSON = {}, On-chain = {} ",
            token.symbol, token.symbol, on_chain_symbol
        );
        local_error_count += 1;
    }

    if token.name != on_chain_name {
        println!(
            "[{}] ❌ Name mismatch: JSON = {}, On-chain = {} ",
            token.symbol, token.name, on_chain_name
        );
        local_error_count += 1;
    }

    if token.decimals != on_chain_decimals {
        println!(
            "[{}] ❌ Decimals mismatch: JSON = {}, On-chain = {} ",
            token.symbol, token.decimals, on_chain_decimals
        );
        local_error_count += 1;
    }

    if local_error_count == 0 {
        println!("[{}] ✅ Token is valid ", token.symbol);
    }

    Ok(local_error_count)
}

fn get_ethereum_provider(network: &str) -> Option<RootProvider<Http<Client>>> {
    let provider_suffix = env::var("RPC_PROVIDER").ok()?;

    let subdomain = match network {
        "ethereum" => "1.ethereum",
        "bob" => "60808.bob",
        "corn" => "21000000.corn",
        _ => return None,
    };

    let full_url = format!("https://rpc.{subdomain}{}", provider_suffix);
    let provider = ProviderBuilder::new().on_http(Url::parse(&full_url).ok()?);

    Some(provider)
}

async fn verify_on_babylon(token: &Token) -> Result<u32, Box<dyn std::error::Error>> {
    let client = Client::new();

    if token.address == "ubbn" {
        println!("ℹ️ Skipping non-contract token: {}", token.symbol);
        return Ok(0);
    }

    if token.address.starts_with("ibc/") {
        println!("ℹ️ Skipping IBC token: {}", token.symbol);
        return Ok(0);
    }

    let msg = serde_json::json!({ "token_info": {} });
    let query = general_purpose::STANDARD.encode(msg.to_string());

    let rpc_url = "https://babylon.nodes.guru";
    let url = format!(
        "{}/api/cosmwasm/wasm/v1/contract/{}/smart/{}",
        rpc_url, token.address, query
    );

    let res: TokenInfoResponse = client.get(&url).send().await?.json().await?;

    let mut local_error_count = 0;

    if token.symbol != res.data.symbol {
        println!(
            "[{}] ❌ Symbol mismatch: JSON = {}, On-chain = {}",
            token.symbol, token.symbol, res.data.symbol
        );
        local_error_count += 1;
    }

    if token.name != res.data.name {
        println!(
            "[{}] ❌ Name mismatch: JSON = {}, On-chain = {}",
            token.symbol, token.name, res.data.name
        );
        local_error_count += 1;
    }

    if token.decimals != res.data.decimals {
        println!(
            "[{}] ❌ Decimals mismatch: JSON = {}, On-chain = {}",
            token.symbol, token.decimals, res.data.decimals
        );
        local_error_count += 1;
    }

    if local_error_count == 0 {
        println!("[{}] ✅ Token is valid ", token.symbol);
    }

    Ok(local_error_count)
}
