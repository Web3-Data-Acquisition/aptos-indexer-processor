use std::collections::{HashMap, HashSet};
use crate::{
    db::common::models::events_models::EventModel,
    schema,
    utils::database::{execute_in_chunks, get_config_table_chunk_size, ArcDbPool},
};
use ahash::AHashMap;
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    traits::{async_step::AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use async_trait::async_trait;
use diesel::{
    pg::{upsert::excluded, Pg},
    query_builder::QueryFragment,
    ExpressionMethods,
};
use redis::{Client, Commands};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use tracing::{error, info};

// 自定义一个事件信息
#[derive(Debug, Serialize, Deserialize, Clone)]
struct EventInfo {
    sequence_number: u64,
    creation_number: u64,
    account_address: String,
    transaction_version: u64,
    transaction_block_height: u64,
    type_: String,
    data: EventData,  
    event_index: u64,
    indexed_type: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct EventData {
    epoch: String,
    failed_proposer_indices: Vec<u32>,  // 假设这是一个 u32 类型的数组
    hash: String,
    height: String,
    previous_block_votes_bitvec: String,
    proposer: String,
    round: String,
    time_microseconds: String,
}


/// EventsStorer is a step that inserts events in the database.
pub struct EventsStorer
where
    Self: Sized + Send + 'static,
{
    conn_pool: ArcDbPool,
}

impl EventsStorer {
    pub fn new(conn_pool: ArcDbPool) -> Self {
        Self { conn_pool }
    }
}

fn insert_events_query(
    items_to_insert: Vec<EventModel>,
) -> (
    impl QueryFragment<Pg> + diesel::query_builder::QueryId + Send,
    Option<&'static str>,
) {
    // 监测指定的类型
    // println!("thsi is items to inser--------------{:?}", items_to_insert);
    // for item in items_to_insert.clone().first() {
    //     println!("{:?}", item.type_);
    // };

    // let exclude_type = "0x1::block::NewBlockEvent"; // 这是你想排除的类型

    // let filtered_items: Vec<_> = items_to_insert
    //     .iter()
    //     .filter(|item| item.indexed_type == exclude_type)
    //     .cloned()
    //     .collect();

    //删选字段中是否存在某些类型
    let exclude_types: HashSet<&str> = [
        "liquidity_pool::SwapEvent",
        "code::PublishPackage",
        "assets_v1::CoinCreationEvent",
        // "account::CoinRegisterEvent",
        "assets_v1::prebook_coin"
    ].iter().cloned().collect();

    let filtered_items: Vec<_> = items_to_insert
        .iter()
        .filter(|item| {
            exclude_types.iter().any(|&exclude_type| item.indexed_type.contains(exclude_type))
        })
        .cloned()
        .collect();

    // let client = redis::Client::open("redis://127.0.0.1/6379/").expect("Faialed to create Redis client");
    // let mut con = client.get_connection().expect("Failed to connect to Redis");

    // let filetered_items_list = filtered_items.clone();
    // for item_info in filetered_items_list.iter() {
        // let _message = item_info.clone();
        // let _: ()=con.publish("aptos-move-price", "111").expect("Failed to publish message");


        // println!("---------------this is item_info---------------: {:?}", item_info);
        // println!("---------------this is item_info---------------: {:?}", item_info.type_);

        // let re  = Regex::new(r"[<,]").unwrap();

        // let parts: Vec<&str> = re.split(item_info.type_).collect();


        // println!("---------------this is item_info---------------: {:?}", item_info.data);
        
        // let data = json!(item_info.data);
        // if let Value::Object(map) = data {
        //     for (key, value) in &map {
        //         if let Value::String(str_value) = value {
        //             println!("{}: {}", key, str_value);  // 打印每个键和对应的字符串值
        //         } else {
        //             println!("{}: (not a string)", key);  // 处理非字符串的情况
        //         }
        //     }
        // } else {
        //     println!("Expected JSON object but found other type.");
        // }
    // }


    use schema::events::dsl::*;
    (
        diesel::insert_into(schema::events::table)  // 指定插入目标表events
            .values(filtered_items)     //要插入的数据
            .on_conflict((transaction_version, event_index))    //处理唯一性约束冲突
            .do_update()    //冲突时更新已存在的记录
            .set((
                inserted_at.eq(excluded(inserted_at)),
                indexed_type.eq(excluded(indexed_type)),
            )),
        None,
    )
}

#[async_trait]
impl Processable for EventsStorer {
    type Input = Vec<EventModel>;
    type Output = Vec<EventModel>;
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        events: TransactionContext<Vec<EventModel>>,
    ) -> Result<Option<TransactionContext<Vec<EventModel>>>, ProcessorError> {
        let per_table_chunk_sizes: AHashMap<String, usize> = AHashMap::new();
        let execute_res = execute_in_chunks(
            self.conn_pool.clone(),
            insert_events_query,
            &events.data,
            get_config_table_chunk_size::<EventModel>("events", &per_table_chunk_sizes),
        )
        .await;
        match execute_res {
            Ok(_) => {
                info!(
                    "Events version [{}, {}] stored successfully",
                    events.metadata.start_version, events.metadata.end_version
                );
            }
            Err(e) => {
                error!("Failed to store events: {:?}", e);
            }
        }
        Ok(Some(events))
    }
}

impl AsyncStep for EventsStorer {}

impl NamedStep for EventsStorer {
    fn name(&self) -> String {
        "EventsStorer".to_string()
    }
}
