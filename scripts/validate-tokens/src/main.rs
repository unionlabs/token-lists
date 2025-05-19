use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::sol;
use alloy::transports::http::reqwest::Url;
use alloy::transports::http::{Client, Http};
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
                verify_on_babylon(&token)?;
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
    let address = Address::from_str(&token.address)?;

    let contract = IERC20::new(address, &provider);

    let on_chain_symbol = contract.symbol().call().await?._0;
    let on_chain_name = contract.name().call().await?._0;

    let on_chain_decimals = contract.decimals().call().await?._0;

    println!("🔗 [{}]", token.symbol);
    if token.symbol != on_chain_symbol {
        println!(
            "❌ Symbol mismatch: JSON = {}, On-chain = {}",
            token.symbol, on_chain_symbol
        );
    }
    if token.name != on_chain_name {
        println!(
            "❌ Name mismatch: JSON = {}, On-chain = {}",
            token.name, on_chain_name
        );
    }
    if token.decimals != on_chain_decimals {
        println!(
            "❌ Decimals mismatch: JSON = {}, On-chain = {}",
            token.decimals, on_chain_decimals
        );
    }

    if token.symbol == on_chain_symbol
        && token.name == on_chain_name
        && token.decimals == on_chain_decimals
    {
        println!("✅ Token is valid.");
    }

    Ok(())
}

fn get_ethereum_provider(network: &str) -> Option<RootProvider<Http<Client>>> {
    let url = match network {
        "ethereum" => "x",
        "bob" => "",
        "corn" => "x",
        _ => return None,
    };

    let provider = ProviderBuilder::new().on_http(Url::parse(url).ok()?);

    Some(provider)
}

// Placeholder: consultar via Cosmos RPC ou outro método específico de Babylon
fn verify_on_babylon(token: &Token) -> Result<(), Box<dyn std::error::Error>> {
    println!("🪐 [Babylon] Verifying {}", token.address);
    // Aqui irás fazer lógica customizada para Babylon
    Ok(())
}
