use anyhow::Result;
use aptos_indexer_processor_example::{config::indexer_processor_config::IndexerProcessorConfig, utils::{aptos_price::get_aptos_price, calculate_coin_price::get_calculate_coin_price, database::DbPoolConnection}};
use aptos_indexer_processor_sdk_server_framework::ServerArgs;
use clap::Parser;

#[cfg(unix)]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() -> Result<()> {
    let num_cpus = num_cpus::get(); // num_cpus = 8
    let worker_threads = (num_cpus).max(16);

    let mut builder = tokio::runtime::Builder::new_multi_thread();

    builder
        .disable_lifo_slot()
        .enable_all()
        .worker_threads(worker_threads)
        .build()
        .unwrap()
        .block_on(async {
            let args = ServerArgs::parse();
            args.run::<IndexerProcessorConfig>(tokio::runtime::Handle::current())
                .await
        })
}


// #[tokio::main]
// async fn main() {
//     match get_aptos_price().await {
//         Some(price) => println!("The current price of Aptos (APT) is: ${:.8} USDT", price),
//         None => println!("Failed to retrieve Aptos price."),
//     }
// }


// #[tokio::main] // 使用tokio作为异步运行时
// async fn main() {
//     let input_value = 6916331.0;
//     let output_value = 50589929.0;

//     // 调用calculate_final_price函数
//     match get_calculate_coin_price(input_value, output_value).await {
//         Some(price) => println!("The calculated price is: {}", price),
//         None => println!("Failed to get the base price"),
//     }
// }