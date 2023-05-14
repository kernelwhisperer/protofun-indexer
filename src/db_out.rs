use substreams::{
    store::{self, DeltaProto},
    Hex,
};
use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};

use crate::pb::sf::ethereum::block_meta::v1::BlockMeta;

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
        .push_change("block_meta", key, ordinal, Operation::Create)
        .change("number", (None, value.number))
        .change("hash", (None, Hex(value.hash)))
        .change("gas_used", (None, value.gas_used))
        .change("base_fee_per_gas", (None, value.base_fee_per_gas))
        .change("timestamp", (None, value.timestamp));
}

fn push_update(
    changes: &mut DatabaseChanges,
    key: &str,
    ordinal: u64,
    old_value: BlockMeta,
    new_value: BlockMeta,
) {
    changes
        .push_change("block_meta", key, ordinal, Operation::Update)
        .change("number", (old_value.number, new_value.number))
        .change("hash", (Hex(old_value.hash), Hex(new_value.hash)))
        .change("gas_used", (old_value.gas_used, new_value.gas_used))
        .change("base_fee_per_gas", (old_value.base_fee_per_gas, new_value.base_fee_per_gas))
        .change("timestamp", (old_value.timestamp, new_value.timestamp));
}
