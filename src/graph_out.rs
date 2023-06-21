use substreams::store::{self, DeltaBigInt, DeltaProto};
use substreams_entity_change::tables::Tables;

use crate::pb::sf::ethereum::block_meta::v1::{BlockMeta, TransactionMeta};
use substreams::{scalar::BigInt, Hex};

pub fn block_meta_to_tables(tables: &mut Tables, deltas: store::Deltas<DeltaProto<BlockMeta>>) {
    use substreams::pb::substreams::store_delta::Operation;

    for delta in deltas.deltas {
        match delta.operation {
            Operation::Create => push_create(tables, delta.new_value),
            Operation::Update => push_update(tables, delta.old_value, delta.new_value),
            Operation::Delete => todo!(),
            x => panic!("unsupported operation {:?}", x),
        }
    }
}

fn push_create(tables: &mut Tables, value: BlockMeta) {
    tables
        .create_row("Block", Hex(value.hash.clone()).to_string())
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

    for tx in value.txns {
        push_tx_meta_create(tables, value.number, value.hash.clone(), &tx);
    }
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

    // Delete transactions from the old block
    for tx in old_value.txns {
        tables.delete_row("transactions", Hex(tx.hash).to_string());
    }

    // Create transactions from the new block
    for tx in new_value.txns {
        push_tx_meta_create(tables, new_value.number, new_value.hash.clone(), &tx);
    }
}

fn push_tx_meta_create(
    tables: &mut Tables,
    block_number: u64,
    block_hash: Vec<u8>,
    tx: &TransactionMeta,
) {
    tables
        .create_row("Txn", Hex(tx.hash.clone()).to_string())
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

pub fn base_fee_per_gas_minute_to_tables(
    tables: &mut Tables,
    open_deltas: store::Deltas<DeltaBigInt>,
    high_deltas: store::Deltas<DeltaBigInt>,
    low_deltas: store::Deltas<DeltaBigInt>,
    close_deltas: store::Deltas<DeltaBigInt>,
) {
    for delta in open_deltas.deltas {
        tables
            .update_row("BaseFeePerGasMinuteCandle", delta.key.clone())
            .set("timestamp", delta.key.parse::<i64>().unwrap())
            .set("open", &delta.new_value);
    }
    for delta in high_deltas.deltas {
        tables
            .update_row("BaseFeePerGasMinuteCandle", delta.key.clone())
            .set("timestamp", delta.key.parse::<i64>().unwrap())
            .set("high", &delta.new_value);
    }
    for delta in low_deltas.deltas {
        tables
            .update_row("BaseFeePerGasMinuteCandle", delta.key.clone())
            .set("timestamp", delta.key.parse::<i64>().unwrap())
            .set("low", &delta.new_value);
    }
    for delta in close_deltas.deltas {
        tables
            .update_row("BaseFeePerGasMinuteCandle", delta.key.clone())
            .set("timestamp", delta.key.parse::<i64>().unwrap())
            .set("close", &delta.new_value);
    }
}
