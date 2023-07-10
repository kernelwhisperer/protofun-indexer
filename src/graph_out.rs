use substreams::{
    scalar::BigDecimal,
    store::{StoreGet, StoreGetBigInt},
};
use substreams_entity_change::tables::Tables;

use crate::pb::sf::ethereum::block_meta::v1::{BlockMeta, TransactionMeta};
use substreams::{scalar::BigInt, Hex};

pub fn block_meta_to_tables(tables: &mut Tables, block_meta: BlockMeta) {
    push_create(tables, block_meta)
}

fn push_create(tables: &mut Tables, value: BlockMeta) {
    tables
        .update_row("Block", Hex(value.hash.clone()).to_string())
        .set("number", value.number)
        .set("gasUsed", value.gas_used)
        .set(
            "baseFeePerGas",
            BigInt::from_unsigned_bytes_be(&value.base_fee_per_gas),
        )
        .set("timestamp", value.timestamp)
        .set("txnCount", value.txn_count)
        .set(
            "minGasPrice",
            BigInt::from_unsigned_bytes_be(&value.min_gas_price),
        )
        .set(
            "maxGasPrice",
            BigInt::from_unsigned_bytes_be(&value.max_gas_price),
        )
        .set(
            "firstGasPrice",
            BigInt::from_unsigned_bytes_be(&value.first_gas_price),
        )
        .set(
            "lastGasPrice",
            BigInt::from_unsigned_bytes_be(&value.last_gas_price),
        )
        .set(
            "burnedFees",
            BigInt::from_unsigned_bytes_be(&value.burned_fees),
        )
        .set("gasFees", BigInt::from_unsigned_bytes_be(&value.gas_fees))
        .set(
            "minerTips",
            BigInt::from_unsigned_bytes_be(&value.miner_tips),
        );

    // for tx in value.txns {
    //     push_tx_meta_create(tables, value.number, value.hash.clone(), &tx);
    // }
}

fn push_update(tables: &mut Tables, old_value: BlockMeta, new_value: BlockMeta) {
    tables
        .update_row("Block", Hex(new_value.hash.clone()).to_string())
        .set("number", new_value.number)
        .set("gasUsed", new_value.gas_used)
        .set("baseFeePerGas", new_value.base_fee_per_gas)
        .set("timestamp", new_value.timestamp)
        .set("txnCount", new_value.txn_count)
        .set("minGasPrice", new_value.min_gas_price)
        .set("maxGasPrice", new_value.max_gas_price)
        .set(
            "firstGasPrice",
            BigInt::from_unsigned_bytes_be(&new_value.first_gas_price),
        )
        .set(
            "lastGasPrice",
            BigInt::from_unsigned_bytes_be(&new_value.last_gas_price),
        )
        .set("burnedFees", new_value.burned_fees)
        .set("gasFees", new_value.gas_fees)
        .set("minerTips", new_value.miner_tips);

    // // Delete transactions from the old block
    // for tx in old_value.txns {
    //     tables.delete_row("transactions", Hex(tx.hash).to_string());
    // }

    // // Create transactions from the new block
    // for tx in new_value.txns {
    //     push_tx_meta_create(tables, new_value.number, new_value.hash.clone(), &tx);
    // }
}

fn push_tx_meta_create(
    tables: &mut Tables,
    block_number: u64,
    block_hash: Vec<u8>,
    tx: &TransactionMeta,
) {
    tables
        .update_row("Txn", Hex(tx.hash.clone()).to_string())
        .set("blockNumber", block_number)
        .set("block", block_hash)
        .set("gasUsed", tx.gas_used)
        .set("gasPrice", BigInt::from_unsigned_bytes_be(&tx.gas_price))
        .set("gasFee", BigInt::from_unsigned_bytes_be(&tx.gas_fee))
        .set("timestamp", tx.timestamp)
        .set("index", tx.index)
        .set("txnType", tx.txn_type)
        .set(
            "maxPriorityFeePerGas",
            BigInt::from_unsigned_bytes_be(&tx.max_priority_fee_per_gas),
        )
        .set("burnedFee", BigInt::from_unsigned_bytes_be(&tx.burned_fee))
        .set("minerTip", BigInt::from_unsigned_bytes_be(&tx.miner_tip));
}

pub fn candle_to_tables(
    tables: &mut Tables,
    entity_id: &str,
    entity_key: String,
    store_open: StoreGetBigInt,
    store_high: StoreGetBigInt,
    store_low: StoreGetBigInt,
    store_close: StoreGetBigInt,
) {
    let row = tables.update_row(entity_id, entity_key.clone());
    row.set("timestamp", entity_key.parse::<i64>().unwrap());

    match store_open.get_last(entity_key.clone()) {
        None => {}
        Some(value) => {
            row.set("open", value);
        }
    }
    match store_high.get_last(entity_key.clone()) {
        None => {}
        Some(value) => {
            row.set("high", value);
        }
    }
    match store_low.get_last(entity_key.clone()) {
        None => {}
        Some(value) => {
            row.set("low", value);
        }
    }
    match store_close.get_last(entity_key.clone()) {
        None => {}
        Some(value) => {
            row.set("close", value);
        }
    }
}
