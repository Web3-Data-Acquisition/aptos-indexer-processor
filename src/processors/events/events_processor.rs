use super::{events_extractor::EventsExtractor, events_storer::EventsStorer};
use crate::{
    common::processor_status_saver::get_processor_status_saver,
    config::indexer_processor_config::IndexerProcessorConfig,
    utils::{
        chain_id::check_or_update_chain_id,
        database::{new_db_pool, run_migrations, ArcDbPool},
        starting_version::get_starting_version,
    },
};
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::{TransactionStream, TransactionStreamConfig},
    builder::ProcessorBuilder,
    common_steps::{
        TransactionStreamStep, VersionTrackerStep, DEFAULT_UPDATE_PROCESSOR_STATUS_SECS,
    },
    traits::IntoRunnableStep,
};
use tracing::info;

pub struct EventsProcessor {
    pub config: IndexerProcessorConfig,
    pub db_pool: ArcDbPool,
}

impl EventsProcessor {
    pub async fn new(config: IndexerProcessorConfig) -> Result<Self> {
        let conn_pool = new_db_pool(
            &config.db_config.postgres_connection_string,
            Some(config.db_config.db_pool_size),
        )
        .await
        .expect("Failed to create connection pool");

        Ok(Self {
            config,
            db_pool: conn_pool,
        })
    }

    pub async fn run_processor(self) -> Result<()> {
        // Run migrations
        run_migrations(
            self.config.db_config.postgres_connection_string.clone(),
            self.db_pool.clone(),
        )
        .await;

        // 合并配置中的起始版本和数据库中最新处理的版本
        let starting_version = get_starting_version(&self.config, self.db_pool.clone()).await?;

        // 检查并更新分类账链 ID，以确保我们索引正确的链
        let grpc_chain_id = TransactionStream::new(self.config.transaction_stream_config.clone())
            .await?
            .get_chain_id()
            .await?;
        check_or_update_chain_id(grpc_chain_id as i64, self.db_pool.clone()).await?;

        // 定义处理器步骤
        let transaction_stream_config = self.config.transaction_stream_config.clone();
        info!("this++++++ is------- a *******transaction{:?}", transaction_stream_config);

        let transaction_stream = TransactionStreamStep::new(TransactionStreamConfig {
            starting_version: Some(starting_version),
            ..transaction_stream_config
        })
        .await?;
        

        let events_extractor = EventsExtractor {};
        let events_storer = EventsStorer::new(self.db_pool.clone());
        let version_tracker = VersionTrackerStep::new(
            get_processor_status_saver(self.db_pool.clone(), self.config.clone()),
            DEFAULT_UPDATE_PROCESSOR_STATUS_SECS,
        );

        


        // Connect processor steps together
        let (_, buffer_receiver) = ProcessorBuilder::new_with_inputless_first_step(
            transaction_stream.into_runnable_step(),
        )
        .connect_to(events_extractor.into_runnable_step(), 10)
        .connect_to(events_storer.into_runnable_step(), 10)
        .connect_to(version_tracker.into_runnable_step(), 10)
        .end_and_return_output_receiver(10);
        
        // let mut count = 0;
        // let max_count = 0; // 限制循环的最大次数

        // (Optional) Parse the results
        loop {
            // if count > max_count {
            //     println!("thsi is max count {} thsi is  count {}", max_count, count);
            //     info!("Reached the maximum number of iterations: {}", max_count);
            //     return Ok(());
            // }
            
            match buffer_receiver.recv().await {
                Ok(txn_context) => {
                    if txn_context.data.is_empty() {
                        continue;
                    }
                    info!(
                        "Finished processing events from versions [{:?}, {:?}]",
                        txn_context.metadata.start_version, txn_context.metadata.end_version,
                    );
                }
                Err(_) => {
                    info!("Channel is closed");
                    return Ok(());
                }
            }
            // count += 1;
        }
    }
}
