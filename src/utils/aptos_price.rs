use reqwest;
use serde_json::Value;


// 通过币安会aptos的实时价格
pub async fn get_aptos_price() -> Option<f64> {
    let url = "https://api.binance.com/api/v3/ticker/price?symbol=APTUSDT";

    let response = reqwest::get(url).await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                // 处理JSON解析的错误
                match resp.json::<Value>().await {
                    Ok(json_data) => {
                        if let Some(price_str) = json_data["price"].as_str() {
                            price_str.parse::<f64>().ok()
                        } else {
                            None // 如果没有"price"字段，返回None
                        }
                    },
                    Err(_) => None, // JSON解析失败
                }
            } else {
                None // HTTP请求未成功
            }
        },
        Err(_) => None, // 网络请求失败
    }
}