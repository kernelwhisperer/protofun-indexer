mod pb;

use pb::sf::ethereum::block_meta::v1::{BlockMeta, TransactionMeta};

use substreams::scalar::BigInt;
use substreams_ethereum::pb::eth;
// use substreams_ethereum::pb::eth::v2::transaction_trace::Type;

#[path = "db_out.rs"]
mod db_out;
#[path = "graph_out.rs"]
mod graph_out;

use substreams::errors::Error;
use substreams::store::{DeltaProto, StoreSetIfNotExistsProto};
use substreams::{prelude::*, store};
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_entity_change::pb::entity::EntityChanges;
use substreams_ethereum::pb::eth::v2::TransactionTrace;

fn map_block_to_meta(block: eth::v2::Block) -> BlockMeta {
    let header = block.header.as_ref().unwrap();

    let base_fee_per_gas_bytes = header
        .base_fee_per_gas
        .as_ref()
        .map(|base_fee| base_fee.bytes.clone())
        .unwrap_or_else(Vec::new);

    let base_fee_per_gas = BigInt::from_unsigned_bytes_be(&base_fee_per_gas_bytes);
    let is_london_fork = base_fee_per_gas_bytes.len() != 0;

    // substreams::log::info!(
    //     "tx len {} block {}",
    //     block.transaction_traces.len(),
    //     block.number
    // );

    let timestamp = header.timestamp.as_ref().unwrap().seconds;

    let mut gas_fees = BigInt::from(0);
    let mut miner_tips = BigInt::from(0);
    let mut burned_fees = BigInt::from(0);
    let mut min_gas_price: BigInt = BigInt::from(0);
    let mut max_gas_price: BigInt = BigInt::from(0);
    let mut comparison_started = false;

    let transactions: Vec<&TransactionTrace> = block.transactions().collect();

    let txns: Vec<TransactionMeta> = transactions
        .into_iter()
        .rev()
        .enumerate()
        .map(|(index, tx)| {
            // let hash = format!("0x{}", Hex(&tx.hash).to_string());
            // substreams::log::info!("tx hash {}", hash);

            // Because of MEV, this can be 0.
            // E.g.: 0x15614894a056159334f52b791611ca49e8874d0494cec1414b39fec1bf4f5156
            let gas_price = BigInt::from_unsigned_bytes_be(
                tx.gas_price.as_ref().map_or(&vec![0], |x| &x.bytes),
            );
            let gas_used = BigInt::from(tx.gas_used);
            let gas_fee = gas_price.clone() * gas_used.clone();

            let burned_fee = if is_london_fork {
                base_fee_per_gas.clone() * gas_used.clone()
            } else {
                BigInt::from(0)
            };

            let max_priority_fee_per_gas = if !is_london_fork {
                gas_price.clone()
            } else if tx.r#type == 2 {
                // TODO: use Type::TrxTypeDynamicFee enum
                BigInt::from_unsigned_bytes_be(
                    &tx.max_priority_fee_per_gas
                        .as_ref()
                        .unwrap_or(&substreams_ethereum::pb::eth::v2::BigInt { bytes: Vec::new() })
                        .bytes,
                )
            } else {
                gas_price.clone() - base_fee_per_gas.clone()
            };

            let miner_tip = if !is_london_fork {
                gas_fee.clone()
            } else {
                gas_used * max_priority_fee_per_gas.clone()
            };

            //
            gas_fees = gas_fees.clone() + gas_fee.clone();
            miner_tips = miner_tips.clone() + miner_tip.clone();
            if is_london_fork {
                burned_fees = burned_fees.clone() + burned_fee.clone()
            }
            if comparison_started == false {
                comparison_started = true;
                min_gas_price = gas_price.clone();
                max_gas_price = gas_price.clone();
            } else {
                if min_gas_price > gas_price {
                    min_gas_price = gas_price.clone();
                }

                if max_gas_price < gas_price {
                    max_gas_price = gas_price.clone();
                }
            }

            TransactionMeta {
                hash: tx.hash.clone(),
                gas_used: tx.gas_used,
                gas_price: gas_price.to_bytes_be().1,
                gas_fee: gas_fee.to_bytes_be().1,
                txn_type: tx.r#type,
                timestamp,
                index: index as i32,
                max_priority_fee_per_gas: max_priority_fee_per_gas.to_bytes_be().1,
                burned_fee: burned_fee.to_bytes_be().1,
                miner_tip: miner_tip.to_bytes_be().1,
            }
        })
        .collect();

    let mut first_gas_price = vec![0];
    let mut last_gas_price = vec![0];

    if txns.len() != 0 {
        first_gas_price = txns[0].gas_price.clone();
        last_gas_price = txns[txns.len() - 1].gas_price.clone();
    }

    BlockMeta {
        hash: block.hash,
        number: block.number,
        timestamp,
        gas_used: header.gas_used,
        base_fee_per_gas: base_fee_per_gas.to_bytes_be().1,
        txns,
        txn_count: block.transaction_traces.len() as i32,
        min_gas_price: min_gas_price.to_bytes_be().1,
        max_gas_price: max_gas_price.to_bytes_be().1,
        first_gas_price,
        last_gas_price,
        burned_fees: burned_fees.to_bytes_be().1,
        gas_fees: gas_fees.to_bytes_be().1,
        miner_tips: miner_tips.to_bytes_be().1,
    }
}

#[substreams::handlers::map]
fn map_block(block: eth::v2::Block) -> Result<BlockMeta, substreams::errors::Error> {
    Ok(map_block_to_meta(block))
}

#[substreams::handlers::store]
fn store_block_meta(block: eth::v2::Block, store: StoreSetIfNotExistsProto<BlockMeta>) {
    let meta: BlockMeta = map_block_to_meta(block);

    store.set_if_not_exists(0, &meta.number.to_string(), &meta);
    store.delete_prefix(0, &(&meta.number - 10_000).to_string());
}

#[substreams::handlers::map]
pub fn db_out(block_meta: store::Deltas<DeltaProto<BlockMeta>>) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();
    db_out::block_meta_to_tables(&mut tables, block_meta);

    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
pub fn graph_out(
    block: eth::v2::Block,
    base_fee_per_gas_minute_open: StoreGetBigInt,
    base_fee_per_gas_minute_high: StoreGetBigInt,
    base_fee_per_gas_minute_low: StoreGetBigInt,
    base_fee_per_gas_minute_close: StoreGetBigInt,
    base_fee_per_gas_hour_open: StoreGetBigInt,
    base_fee_per_gas_hour_high: StoreGetBigInt,
    base_fee_per_gas_hour_low: StoreGetBigInt,
    base_fee_per_gas_hour_close: StoreGetBigInt,
    base_fee_per_gas_day_open: StoreGetBigInt,
    base_fee_per_gas_day_high: StoreGetBigInt,
    base_fee_per_gas_day_low: StoreGetBigInt,
    base_fee_per_gas_day_close: StoreGetBigInt,
    base_fee_per_gas_week_open: StoreGetBigInt,
    base_fee_per_gas_week_high: StoreGetBigInt,
    base_fee_per_gas_week_low: StoreGetBigInt,
    base_fee_per_gas_week_close: StoreGetBigInt,
) -> Result<EntityChanges, Error> {
    let block_meta = map_block_to_meta(block);
    let mut tables = substreams_entity_change::tables::Tables::new();

    graph_out::block_meta_to_tables(&mut tables, block_meta.clone());
    graph_out::candle_to_tables(
        &mut tables,
        "BaseFeePerGasMinuteCandle",
        get_latest_time_unit(block_meta.timestamp, 60, 0).to_string(),
        base_fee_per_gas_minute_open,
        base_fee_per_gas_minute_high,
        base_fee_per_gas_minute_low,
        base_fee_per_gas_minute_close,
    );
    graph_out::candle_to_tables(
        &mut tables,
        "BaseFeePerGasHourCandle",
        get_latest_time_unit(block_meta.timestamp, 3600, 0).to_string(),
        base_fee_per_gas_hour_open,
        base_fee_per_gas_hour_high,
        base_fee_per_gas_hour_low,
        base_fee_per_gas_hour_close,
    );
    graph_out::candle_to_tables(
        &mut tables,
        "BaseFeePerGasDayCandle",
        get_latest_time_unit(block_meta.timestamp, 86400, 0).to_string(),
        base_fee_per_gas_day_open,
        base_fee_per_gas_day_high,
        base_fee_per_gas_day_low,
        base_fee_per_gas_day_close,
    );
    graph_out::candle_to_tables(
        &mut tables,
        "BaseFeePerGasWeekCandle",
        get_latest_time_unit(block_meta.timestamp, 604800, 345600).to_string(),
        base_fee_per_gas_week_open,
        base_fee_per_gas_week_high,
        base_fee_per_gas_week_low,
        base_fee_per_gas_week_close,
    );

    Ok(tables.to_entity_changes())
}

fn get_latest_time_unit(timestamp: i64, interval_in_seconds: i64, offset: i64) -> i64 {
    let timestamp_seconds = timestamp;
    let latest_time_unit =
        ((timestamp_seconds - offset) / interval_in_seconds) * interval_in_seconds;

    return latest_time_unit + offset;
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_minute_open(block_meta: BlockMeta, store: StoreSetIfNotExistsBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 60, 0);
    store.set_if_not_exists(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_minute_low(block_meta: BlockMeta, store: StoreMinBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 60, 0);
    store.min(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_minute_high(block_meta: BlockMeta, store: StoreMaxBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 60, 0);
    store.max(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_minute_close(block_meta: BlockMeta, store: StoreSetBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 60, 0);
    store.set(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_hour_open(block_meta: BlockMeta, store: StoreSetIfNotExistsBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 3600, 0);
    store.set_if_not_exists(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_hour_low(block_meta: BlockMeta, store: StoreMinBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 3600, 0);
    store.min(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_hour_high(block_meta: BlockMeta, store: StoreMaxBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 3600, 0);
    store.max(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_hour_close(block_meta: BlockMeta, store: StoreSetBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 3600, 0);
    store.set(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_day_open(block_meta: BlockMeta, store: StoreSetIfNotExistsBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 86400, 0);
    store.set_if_not_exists(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_day_low(block_meta: BlockMeta, store: StoreMinBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 86400, 0);
    store.min(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_day_high(block_meta: BlockMeta, store: StoreMaxBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 86400, 0);
    store.max(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_day_close(block_meta: BlockMeta, store: StoreSetBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 86400, 0);
    store.set(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 172800).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_week_open(block_meta: BlockMeta, store: StoreSetIfNotExistsBigInt) {
    // 1688958887 should give 1688947200
    // 1628166822 should give 1627862400
    let id = get_latest_time_unit(block_meta.timestamp, 604800, 345600);
    store.set_if_not_exists(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 1209600).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_week_low(block_meta: BlockMeta, store: StoreMinBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 604800, 345600);
    store.min(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 1209600).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_week_high(block_meta: BlockMeta, store: StoreMaxBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 604800, 345600);
    store.max(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 1209600).to_string());
}

#[substreams::handlers::store]
fn store_base_fee_per_gas_week_close(block_meta: BlockMeta, store: StoreSetBigInt) {
    let id = get_latest_time_unit(block_meta.timestamp, 604800, 345600);
    store.set(
        0,
        id.to_string(),
        &BigInt::from_unsigned_bytes_be(&block_meta.base_fee_per_gas),
    );
    store.delete_prefix(0, &(id - 1209600).to_string());
}
