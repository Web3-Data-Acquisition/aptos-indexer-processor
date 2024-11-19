// use reqwest;
// use serde::{Serialize, Deserialize};
// use serde_json::Value;

// #[derive(Debug, Serialize, Deserialize)]
// struct Filter {
//     filterType: String,
//     minQty: Option<String>,
//     stepSize: Option<String>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Symbol {
//     symbol: String,
//     filters: Vec<Filter>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct ExchangeInfo {
//     symbols: Vec<Symbol>,
// }

// #[derive(Debug)]
// struct AptosLotSizeInfo {
//     symbol: String,
//     minimum_quantity: String,
//     step_size: String,
// }


// // 获取aptos的币对精度
// pub async fn get_aptos_decimals() -> Option<AptosLotSizeInfo> {
//     let url = "https://api.binance.com/api/v3/exchangeInfo";

//     let client = reqwest::Client::new();
//     let response = match client.get(url).send().await {
//         Ok(res) => res,
//         Err(_) => return None,
//     };

//     let data: ExchangeInfo = match response.json().await {
//         Ok(data) => data,
//         Err(_) => return None,
//     };

//     for symbol in data.symbols {
//         if symbol.symbol.contains("APT") {
//             for filter in symbol.gilters {
//                 if filter.filterType == "LOT_SIZE" {
//                     return Ok(AptosLotSizeInfo {
//                         symbol: symbol.symbol.clone(),
//                         minimum_quantity: filter.minQty.clone().unwrap_or_default(),
//                         step_size: filter.stepSize.clone().unwrap_or_default(),  
//                     });
//                 }
//             }
//         }
//     }
//     None
// }