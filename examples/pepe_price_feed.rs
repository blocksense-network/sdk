use blocksense_sdk::oracle::*;
use blocksense_sdk::oracle_component;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

async fn fetch_pepe_price_from_mock_exchange(exchange_id: &str) -> Result<f64, String> {
    let mock_prices = match exchange_id {
        "binance_mock" => 0.00002156,
        "coinbase_mock" => 0.00002143,
        "kraken_mock" => 0.00002168,
        "bybit_mock" => 0.00002151,
        "okx_mock" => 0.00002162,
        _ => return Err(format!("Unknown exchange: {}", exchange_id)),
    };

    let mut hasher = DefaultHasher::new();
    exchange_id.hash(&mut hasher);
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    
    let jitter = ((hasher.finish() as f64) / (u64::MAX as f64) - 0.5) * 0.000001;
    Ok(mock_prices + jitter)
}

async fn calculate_aggregated_price(exchanges: &[&str]) -> Result<f64, String> {
    let mut prices = Vec::new();
    let mut successful_fetches = 0;

    for exchange in exchanges {
        match fetch_pepe_price_from_mock_exchange(exchange).await {
            Ok(price) => {
                prices.push(price);
                successful_fetches += 1;
            }
            Err(e) => {
                eprintln!("Failed to fetch from {}: {}", exchange, e);
            }
        }
    }

    if successful_fetches < 3 {
        return Err("Insufficient data sources for reliable price".to_string());
    }

    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_price = if prices.len() % 2 == 0 {
        (prices[prices.len() / 2 - 1] + prices[prices.len() / 2]) / 2.0
    } else {
        prices[prices.len() / 2]
    };

    Ok(median_price)
}

#[oracle_component]
async fn handle_pepe_price_feed(settings: Settings) -> Result<Payload, String> {
    let mut payload = Payload::new();

    let exchanges = ["binance_mock", "coinbase_mock", "kraken_mock", "bybit_mock", "okx_mock"];
    
    for data_feed in &settings.data_feeds {
        match data_feed.id.as_str() {
            "pepe_usd_price" => {
                match calculate_aggregated_price(&exchanges).await {
                    Ok(price) => {
                        payload.values.push(DataFeedResult {
                            id: data_feed.id.clone(),
                            value: DataFeedResultValue::Numerical(price),
                        });
                    }
                    Err(e) => {
                        payload.values.push(DataFeedResult {
                            id: data_feed.id.clone(),
                            value: DataFeedResultValue::Error(e),
                        });
                    }
                }
            }
            "pepe_volume_24h" => {
                let mut hasher = DefaultHasher::new();
                "volume".hash(&mut hasher);
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
                    .hash(&mut hasher);
                
                let volume_jitter = ((hasher.finish() as f64) / (u64::MAX as f64) - 0.5) * 50000000.0;
                let mock_volume = 1250000000.0 + volume_jitter;
                
                payload.values.push(DataFeedResult {
                    id: data_feed.id.clone(),
                    value: DataFeedResultValue::Numerical(mock_volume),
                });
            }
            "pepe_market_status" => {
                payload.values.push(DataFeedResult {
                    id: data_feed.id.clone(),
                    value: DataFeedResultValue::Text("active".to_string()),
                });
            }
            _ => {
                payload.values.push(DataFeedResult {
                    id: data_feed.id.clone(),
                    value: DataFeedResultValue::Error("Unknown data feed ID".to_string()),
                });
            }
        }
    }

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_pepe_price_from_known_exchange() {
        let result = fetch_pepe_price_from_mock_exchange("binance_mock").await;
        assert!(result.is_ok());
        let price = result.unwrap();
        assert!(price > 0.000020 && price < 0.000025);
    }

    #[tokio::test]
    async fn test_fetch_pepe_price_from_unknown_exchange() {
        let result = fetch_pepe_price_from_mock_exchange("unknown_exchange").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown exchange"));
    }

    #[tokio::test]
    async fn test_calculate_aggregated_price() {
        let exchanges = ["binance_mock", "coinbase_mock", "kraken_mock"];
        let result = calculate_aggregated_price(&exchanges).await;
        assert!(result.is_ok());
        let price = result.unwrap();
        assert!(price > 0.000020 && price < 0.000025);
    }

    #[tokio::test]
    async fn test_calculate_aggregated_price_insufficient_sources() {
        let exchanges = ["unknown1", "unknown2"];
        let result = calculate_aggregated_price(&exchanges).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient data sources"));
    }

    #[tokio::test]
    async fn test_handle_pepe_price_feed_with_pepe_usd_price() {
        let settings = Settings::new(
            vec![DataFeedSetting {
                id: "pepe_usd_price".to_string(),
                data: "".to_string(),
            }],
            vec![],
        );

        let result = handle_pepe_price_feed(settings).await;
        assert!(result.is_ok());
        
        let payload = result.unwrap();
        assert_eq!(payload.values.len(), 1);
        assert_eq!(payload.values[0].id, "pepe_usd_price");
        
        match &payload.values[0].value {
            DataFeedResultValue::Numerical(price) => {
                assert!(price > &0.000020 && price < &0.000025);
            }
            _ => panic!("Expected numerical value for PEPE price"),
        }
    }

    #[tokio::test]
    async fn test_handle_pepe_price_feed_with_volume() {
        let settings = Settings::new(
            vec![DataFeedSetting {
                id: "pepe_volume_24h".to_string(),
                data: "".to_string(),
            }],
            vec![],
        );

        let result = handle_pepe_price_feed(settings).await;
        assert!(result.is_ok());
        
        let payload = result.unwrap();
        assert_eq!(payload.values.len(), 1);
        assert_eq!(payload.values[0].id, "pepe_volume_24h");
        
        match &payload.values[0].value {
            DataFeedResultValue::Numerical(volume) => {
                assert!(volume > &1200000000.0 && volume < &1400000000.0);
            }
            _ => panic!("Expected numerical value for PEPE volume"),
        }
    }

    #[tokio::test]
    async fn test_handle_pepe_price_feed_with_market_status() {
        let settings = Settings::new(
            vec![DataFeedSetting {
                id: "pepe_market_status".to_string(),
                data: "".to_string(),
            }],
            vec![],
        );

        let result = handle_pepe_price_feed(settings).await;
        assert!(result.is_ok());
        
        let payload = result.unwrap();
        assert_eq!(payload.values.len(), 1);
        assert_eq!(payload.values[0].id, "pepe_market_status");
        
        match &payload.values[0].value {
            DataFeedResultValue::Text(status) => {
                assert_eq!(status, "active");
            }
            _ => panic!("Expected text value for market status"),
        }
    }

    #[tokio::test]
    async fn test_handle_pepe_price_feed_with_unknown_feed() {
        let settings = Settings::new(
            vec![DataFeedSetting {
                id: "unknown_feed".to_string(),
                data: "".to_string(),
            }],
            vec![],
        );

        let result = handle_pepe_price_feed(settings).await;
        assert!(result.is_ok());
        
        let payload = result.unwrap();
        assert_eq!(payload.values.len(), 1);
        assert_eq!(payload.values[0].id, "unknown_feed");
        
        match &payload.values[0].value {
            DataFeedResultValue::Error(err) => {
                assert!(err.contains("Unknown data feed ID"));
            }
            _ => panic!("Expected error value for unknown feed"),
        }
    }

    #[tokio::test]
    async fn test_handle_pepe_price_feed_multiple_feeds() {
        let settings = Settings::new(
            vec![
                DataFeedSetting {
                    id: "pepe_usd_price".to_string(),
                    data: "".to_string(),
                },
                DataFeedSetting {
                    id: "pepe_volume_24h".to_string(),
                    data: "".to_string(),
                },
                DataFeedSetting {
                    id: "pepe_market_status".to_string(),
                    data: "".to_string(),
                },
            ],
            vec![],
        );

        let result = handle_pepe_price_feed(settings).await;
        assert!(result.is_ok());
        
        let payload = result.unwrap();
        assert_eq!(payload.values.len(), 3);
        
        let feed_ids: Vec<&String> = payload.values.iter().map(|v| &v.id).collect();
        assert!(feed_ids.contains(&&"pepe_usd_price".to_string()));
        assert!(feed_ids.contains(&&"pepe_volume_24h".to_string()));
        assert!(feed_ids.contains(&&"pepe_market_status".to_string()));
    }
}