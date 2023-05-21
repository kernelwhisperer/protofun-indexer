use std::cmp::min;

use substreams::store::{self, DeltaProto};
use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};

use crate::pb::sf::ethereum::block_meta::v1::{BlockMeta, TransactionMeta};

// TESTME: with snapshots
pub fn block_meta_to_database_changes(
    changes: &mut DatabaseChanges,
    deltas: store::Deltas<DeltaProto<BlockMeta>>,
) {
    use substreams::pb::substreams::store_delta::Operation;

    for delta in deltas.deltas {
        match delta.operation {
            Operation::Create => push_create(changes, &delta.key, delta.ordinal, delta.new_value),
            Operation::Update => push_update(
                changes,
                &delta.key,
                delta.ordinal,
                delta.old_value,
                delta.new_value,
            ),
            Operation::Delete => panic!("delete should not happen"),
            x => panic!("unsupported operation {:?}", x),
        }
    }
}

fn push_create(changes: &mut DatabaseChanges, key: &str, ordinal: u64, value: BlockMeta) {
    changes
        .push_change("blocks", key, ordinal, Operation::Create)
        .change("hash", (None, value.hash))
        .change("gas_used", (None, value.gas_used))
        .change("base_fee_per_gas", (None, value.base_fee_per_gas))
        .change("timestamp", (None, value.timestamp))
        .change("min_gas_price", (None, value.min_gas_price))
        .change("max_gas_price", (None, value.max_gas_price))
        .change("burned_fees", (None, value.burned_fees))
        .change("gas_fees", (None, value.gas_fees))
        .change("miner_tips", (None, value.miner_tips));

    for (index, tx) in value.transactions.iter().enumerate() {
        let tx_id = format!("{}_{}", value.number, index);
        push_tx_meta_create(changes, value.number, tx, &tx_id, ordinal);
    }
}

fn push_update(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    old_value: BlockMeta,
    new_value: BlockMeta,
) {
    changes
        .push_change("blocks", key, ordinal, Operation::Update)
        .change("hash", (old_value.hash, new_value.hash))
        .change("gas_used", (old_value.gas_used, new_value.gas_used))
        .change(
            "base_fee_per_gas",
            (old_value.base_fee_per_gas, new_value.base_fee_per_gas),
        )
        .change("timestamp", (old_value.timestamp, new_value.timestamp))
        .change(
            "min_gas_price",
            (old_value.min_gas_price, new_value.min_gas_price),
        )
        .change(
            "max_gas_price",
            (old_value.max_gas_price, new_value.max_gas_price),
        )
        .change(
            "burned_fees",
            (old_value.burned_fees, new_value.burned_fees),
        )
        .change("gas_fees", (old_value.gas_fees, new_value.gas_fees))
        .change("miner_tips", (old_value.miner_tips, new_value.miner_tips));

    for (index, (old_tx, new_tx)) in old_value
        .transactions
        .iter()
        .zip(new_value.transactions.iter())
        .enumerate()
    {
        let tx_id = format!("{}_{}", new_value.number, index);
        push_tx_meta_update(changes, old_tx, new_tx, &tx_id, ordinal);
    }

    // This would be the count of transactions that were zipped and processed
    let processed_count = min(old_value.transactions.len(), new_value.transactions.len());

    // Delete extra transactions from the old block
    for tx_index in processed_count..old_value.transactions.len() {
        let id = format!("{}_{}", old_value.number, tx_index);
        changes.push_change("transactions", &id, ordinal, Operation::Delete);
    }

    // Create additional transactions in the new block
    for tx_index in processed_count..new_value.transactions.len() {
        let tx = &new_value.transactions[tx_index];
        let tx_id = format!("{}_{}", new_value.number, tx_index);
        push_tx_meta_create(changes, new_value.number, tx, &tx_id, ordinal);
    }
}

fn push_tx_meta_create(
    changes: &mut DatabaseChanges,
    block_number: u64,
    tx: &TransactionMeta,
    tx_id: &str,
    ordinal: u64,
) {
    changes
        .push_change("transactions", tx_id, ordinal, Operation::Create)
        .change("gas_used", (None, tx.gas_used))
        .change("gas_price", (None, tx.gas_price.clone()))
        .change("gas_fee", (None, tx.gas_fee.clone()))
        .change("txn_type", (None, tx.txn_type))
        .change(
            "max_priority_fee_per_gas",
            (None, tx.max_priority_fee_per_gas.clone()),
        )
        .change("burned_fee", (None, tx.burned_fee.clone()))
        .change("block_number", (None, block_number))
        .change("hash", (None, tx.hash.clone()))
        .change("miner_tip", (None, tx.miner_tip.clone()));
}

fn push_tx_meta_update(
    changes: &mut DatabaseChanges,
    old_tx: &TransactionMeta,
    new_tx: &TransactionMeta,
    tx_id: &str,
    ordinal: u64,
) {
    changes
        .push_change("transactions", &tx_id, ordinal, Operation::Update)
        .change("gas_used", (Some(old_tx.gas_used), new_tx.gas_used))
        .change(
            "gas_price",
            (Some(old_tx.gas_price.clone()), new_tx.gas_price.clone()),
        )
        .change(
            "gas_fee",
            (Some(old_tx.gas_fee.clone()), new_tx.gas_fee.clone()),
        )
        .change("txn_type", (Some(old_tx.txn_type), new_tx.txn_type))
        .change(
            "max_priority_fee_per_gas",
            (
                Some(old_tx.max_priority_fee_per_gas.clone()),
                new_tx.max_priority_fee_per_gas.clone(),
            ),
        )
        .change(
            "burned_fee",
            (Some(old_tx.burned_fee.clone()), new_tx.burned_fee.clone()),
        )
        .change(
            "miner_tip",
            (Some(old_tx.miner_tip.clone()), new_tx.miner_tip.clone()),
        );
}
