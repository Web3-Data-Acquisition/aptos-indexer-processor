use super::database::ArcDbPool;
use crate::{
    config::indexer_processor_config::IndexerProcessorConfig,
    db::common::models::{
        backfill_processor_status::BackfillProcessorStatusQuery,
        processor_status::ProcessorStatusQuery,
    },
};
use anyhow::{Context, Result};

pub async fn get_starting_version(
    indexer_processor_config: &IndexerProcessorConfig,
    conn_pool: ArcDbPool,
) -> Result<u64> {
    // 如果在 TransactionStreamConfig 中设置了 Starting_version，则使用该
    if indexer_processor_config
        .transaction_stream_config
        .starting_version
        .is_some()
    {
        return Ok(indexer_processor_config
            .transaction_stream_config
            .starting_version
            .unwrap());
    }

    // 如果未设置，请检查数据库是否设置了 latest_processed_version 并使用该版本
    let latest_processed_version_from_db =
        get_latest_processed_version_from_db(indexer_processor_config, conn_pool)
            .await
            .context("Failed to get latest processed version from DB")?;
    
    println!("从数据库获取的最新处理版本号： {:?}", latest_processed_version_from_db);

    if let Some(latest_processed_version_tracker) = latest_processed_version_from_db {
        return Ok(latest_processed_version_tracker + 1);
    }

    // 如果 latest_processed_version 没有存储在 DB 中，则返回默认的 0
    Ok(0)
}

/// 获取处理器的起始版本。如果未找到，则从 0 开始。
pub async fn get_latest_processed_version_from_db(
    indexer_processor_config: &IndexerProcessorConfig,
    conn_pool: ArcDbPool,
) -> Result<Option<u64>> {
    let mut conn = conn_pool.get().await?;

    if let Some(backfill_config) = &indexer_processor_config.backfill_config {
        return match BackfillProcessorStatusQuery::get_by_processor(
            &backfill_config.backfill_alias,
            &mut conn,
        )
        .await?
        {
            Some(status) => Ok(Some(status.last_success_version as u64)),
            None => Ok(None),
        };
    }

    match ProcessorStatusQuery::get_by_processor(
        indexer_processor_config.processor_config.name(),
        &mut conn,
    )
    .await?
    {
        Some(status) => Ok(Some(status.last_success_version as u64)),
        None => Ok(None),
    }
}
