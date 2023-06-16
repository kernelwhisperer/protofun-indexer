use substreams::store::{self, DeltaProto};
use substreams_entity_change::tables::Tables;

use crate::pb::sf::ethereum::block_meta::v1::{BlockMeta, TransactionMeta};

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
        .create_row("Block", value.hash.clone())
        .set("number", value.number)
        .set("gasUsed", value.gas_used)
        .set("baseFeePerGas", value.base_fee_per_gas)
        .set("timestamp", value.timestamp)
        .set("minGasPrice", value.min_gas_price)
        .set("maxGasPrice", value.max_gas_price)
        .set("burnedFees", value.burned_fees)
        .set("gasFees", value.gas_fees)
        .set("minerTips", value.miner_tips);

    for tx in value.transactions {
        push_tx_meta_create(tables, value.number, value.hash.clone(), &tx);
    }
}

fn push_update(tables: &mut Tables, old_value: BlockMeta, new_value: BlockMeta) {
    tables
        .update_row("Block", new_value.hash.clone())
        .set("number", new_value.number)
        .set("gasUsed", new_value.gas_used)
        .set("baseFeePerGas", new_value.base_fee_per_gas)
        .set("timestamp", new_value.timestamp)
        .set("minGasPrice", new_value.min_gas_price)
        .set("maxGasPrice", new_value.max_gas_price)
        .set("burnedFees", new_value.burned_fees)
        .set("gasFees", new_value.gas_fees)
        .set("minerTips", new_value.miner_tips);

    // Delete transactions from the old block
    for tx in old_value.transactions {
        tables.delete_row("transactions", tx.hash.clone());
    }

    // Create transactions from the new block
    for tx in new_value.transactions {
        push_tx_meta_create(tables, new_value.number, new_value.hash.clone(), &tx);
    }
}

fn push_tx_meta_create(
    tables: &mut Tables,
    block_number: u64,
    block_hash: String,
    tx: &TransactionMeta,
) {
    tables
        .create_row("Txn", tx.hash.clone())
        .set("blockNumber", block_number)
        .set("block", block_hash)
        .set("gasUsed", tx.gas_used)
        .set("gasPrice", tx.gas_price.clone())
        .set("gasFee", tx.gas_fee.clone())
        .set("txnType", tx.txn_type)
        .set(
            "maxPriorityFeePerGas",
            tx.max_priority_fee_per_gas.clone(),
        )
        .set("burnedFee", tx.burned_fee.clone())
        .set("minerTip", tx.miner_tip.clone());
}
