use substreams::store::{self, DeltaProto};
use substreams_database_change::tables::Tables;

use crate::pb::sf::ethereum::block_meta::v1::{BlockMeta, TransactionMeta};
use substreams::{scalar::BigInt, Hex};

// TESTME: with snapshots
pub fn block_meta_to_tables(tables: &mut Tables, deltas: store::Deltas<DeltaProto<BlockMeta>>) {
    use substreams::pb::substreams::store_delta::Operation;

    for delta in deltas.deltas {
        match delta.operation {
            Operation::Create => push_create(tables, &delta.key, delta.new_value),
            Operation::Update => push_update(tables, &delta.key, delta.old_value, delta.new_value),
            Operation::Delete => panic!("delete should not happen"),
            x => panic!("unsupported operation {:?}", x),
        }
    }
}

fn push_create(tables: &mut Tables, key: &str, value: BlockMeta) {
    tables
        .create_row("blocks", key)
        .set("hash", value.hash)
        .set("gas_used", value.gas_used)
        .set("base_fee_per_gas", value.base_fee_per_gas)
        .set("timestamp", value.timestamp)
        .set("min_gas_price", value.min_gas_price)
        .set("max_gas_price", value.max_gas_price)
        .set("burned_fees", value.burned_fees)
        .set("gas_fees", value.gas_fees)
        .set("miner_tips", value.miner_tips);

    for tx in value.transactions {
        push_tx_meta_create(tables, value.number, &tx);
    }
}

fn push_update(tables: &mut Tables, key: &str, old_value: BlockMeta, new_value: BlockMeta) {
    tables
        .update_row("blocks", key)
        .set("hash", new_value.hash)
        .set("gas_used", new_value.gas_used)
        .set("base_fee_per_gas", new_value.base_fee_per_gas)
        .set("timestamp", new_value.timestamp)
        .set("min_gas_price", new_value.min_gas_price)
        .set("max_gas_price", new_value.max_gas_price)
        .set("burned_fees", new_value.burned_fees)
        .set("gas_fees", new_value.gas_fees)
        .set("miner_tips", new_value.miner_tips);

    // Delete transactions from the old block
    for tx in old_value.transactions {
        tables.delete_row("transactions", Hex(tx.hash).to_string());
    }

    // Create transactions from the new block
    for tx in new_value.transactions {
        push_tx_meta_create(tables, new_value.number, &tx);
    }
}

fn push_tx_meta_create(tables: &mut Tables, block_number: u64, tx: &TransactionMeta) {
    tables
        .create_row("transactions", Hex(tx.hash.clone()).to_string())
        .set("hash", tx.hash.clone())
        .set("block_number", block_number)
        .set("gas_used", tx.gas_used)
        .set("gas_price", tx.gas_price.clone())
        .set("gas_fee", tx.gas_fee.clone())
        .set("txn_type", tx.txn_type)
        .set(
            "max_priority_fee_per_gas",
            tx.max_priority_fee_per_gas.clone(),
        )
        .set("burned_fee", tx.burned_fee.clone())
        .set("miner_tip", tx.miner_tip.clone());
}
