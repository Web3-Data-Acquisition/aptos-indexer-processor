use super::aptos_price::get_aptos_price;


pub async fn get_calculate_coin_price(
    input_num: f64, output_num: f64
) -> Option<f64> {
    match get_aptos_price().await {
        Some(aptos_price) => {
            let final_price = input_num * aptos_price / output_num;
            Some(final_price)
        },
        None => None,
    }
}