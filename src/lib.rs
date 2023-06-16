mod pb;

use pb::sf::ethereum::block_meta::v1::{BlockMeta, TransactionMeta};

use substreams::{scalar::BigInt, Hex};
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

fn map_block_to_meta(block: eth::v2::Block) -> BlockMeta {
    let header = block.header.as_ref().unwrap();

    let base_fee_per_gas_bytes = header
        .base_fee_per_gas
        .as_ref()
        .map(|base_fee| base_fee.bytes.clone())
        .unwrap_or_else(Vec::new);

    let base_fee_per_gas = BigInt::from_unsigned_bytes_be(&base_fee_per_gas_bytes);
    let is_london_fork = base_fee_per_gas_bytes.len() != 0;

    let base_fee_per_gas_string = if is_london_fork {
        base_fee_per_gas.to_string()
    } else {
        "".to_string()
    };

    substreams::log::info!("tx len {} block {}", block.transaction_traces.len(), block.number);

    let mut gas_fees = BigInt::from(0);
    let mut miner_tips = BigInt::from(0);
    let mut burned_fees = BigInt::from(0);
    let mut min_gas_price: BigInt = BigInt::from(0);
    let mut max_gas_price: BigInt = BigInt::from(0);
    let mut comparison_started = false;

    let transactions: Vec<TransactionMeta> = block
        .transactions()
        .map(|tx| {
            let hash = Hex(&tx.hash).to_string();
            substreams::log::info!("tx hash {}", hash);

            // Because of MEV, this can be 0.
            // E.g.: 0x15614894a056159334f52b791611ca49e8874d0494cec1414b39fec1bf4f5156
            let gas_price = BigInt::from_unsigned_bytes_be(
                tx.gas_price.as_ref().map_or(&vec![0], |x| &x.bytes),
            );
            let gas_price_str = gas_price.to_string();
            let gas_used = BigInt::from(tx.gas_used);
            let gas_fee = gas_price.clone() * gas_used.clone();

            let burned_fee = if is_london_fork {
                base_fee_per_gas.clone() * gas_used.clone()
            } else {
                BigInt::from(0)
            };
            let burned_fee_str = if is_london_fork {
                burned_fee.to_string()
            } else {
                "".to_string()
            };

            let max_priority_fee_per_gas = if !is_london_fork {
                gas_price.clone()
            } else if tx.r#type == 2 {
                // TODO: use Type::TrxTypeDynamicFee enum
                BigInt::from_unsigned_bytes_be(&tx.max_priority_fee_per_gas.as_ref().unwrap_or(&substreams_ethereum::pb::eth::v2::BigInt{
                    bytes: Vec::new()
                }).bytes)
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
                burned_fees = burned_fees.clone() + burned_fee
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
                hash,
                gas_used: tx.gas_used,
                gas_price: gas_price_str,
                gas_fee: gas_fee.to_string(),
                txn_type: tx.r#type,
                max_priority_fee_per_gas: max_priority_fee_per_gas.to_string(),
                burned_fee: burned_fee_str,
                miner_tip: miner_tip.to_string(),
            }
        })
        .collect();

    BlockMeta {
        hash: Hex(&block.hash).to_string(),
        number: block.number,
        timestamp: header.timestamp.as_ref().unwrap().to_string(),
        gas_used: header.gas_used,
        base_fee_per_gas: base_fee_per_gas_string, 
        transactions,
        min_gas_price: min_gas_price.to_string(), // TODO: move from string to bytes
        max_gas_price: max_gas_price.to_string(),
        burned_fees: burned_fees.to_string(),
        gas_fees: gas_fees.to_string(),
        miner_tips: miner_tips.to_string(),
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
}

#[substreams::handlers::map]
pub fn db_out(block_meta: store::Deltas<DeltaProto<BlockMeta>>) -> Result<DatabaseChanges, Error> {
    let mut database_changes: DatabaseChanges = Default::default();
    db_out::block_meta_to_database_changes(&mut database_changes, block_meta);

    Ok(database_changes)
}

#[substreams::handlers::map]
pub fn graph_out(block_meta: store::Deltas<DeltaProto<BlockMeta>>) -> Result<EntityChanges, Error> {
    let mut tables = substreams_entity_change::tables::Tables::new();
    graph_out::block_meta_to_entities_changes(&mut tables, block_meta);

    Ok(tables.to_entity_changes())
}
