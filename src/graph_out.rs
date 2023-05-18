use substreams::store::{self, DeltaProto};
use substreams_entity_change::pb::entity::{entity_change::Operation, EntityChanges};
mod pb;

use crate::pb::sf::ethereum::block_meta::v1::BlockMeta;

pub fn block_meta_to_entities_changes(
    changes: &mut EntityChanges,
    deltas: store::Deltas<DeltaProto<BlockMeta>>,
) {
    use substreams::pb::substreams::store_delta::Operation;

    for delta in deltas.deltas {
        match delta.operation {
            Operation::Create => push_create(
                changes,
                &delta.key,
                delta.ordinal,
                delta.new_value,
            ),
            Operation::Update => push_update(
                changes,
                &delta.key,
                delta.ordinal,
                delta.old_value,
                delta.new_value,
            ),
            Operation::Delete => todo!(),
            x => panic!("unsupported operation {:?}", x),
        }
    }
}

fn push_create(
    changes: &mut EntityChanges,
    key: &str,
    ordinal: u64,
    value: BlockMeta,
) {
    changes
        .push_change("BlockMeta", key, ordinal, Operation::Create)
        .change("number", value.number)
        .change("hash", value.hash)
        .change("gas_used", value.gas_used)
        .change("base_fee_per_gas", value.base_fee_per_gas)
        .change("timestamp", value.timestamp);
}

fn push_update(
    changes: &mut EntityChanges,
    key: &str,
    ordinal: u64,
    old_value: BlockMeta,
    new_value: BlockMeta,
) {
    changes
        .push_change("BlockMeta", key, ordinal, Operation::Update)
        .change("number", (old_value.number, new_value.number))
        .change("hash", (old_value.hash, new_value.hash))
        .change("gas_used", (old_value.gas_used, new_value.gas_used))
        .change(
            "base_fee_per_gas",
            (old_value.base_fee_per_gas, new_value.base_fee_per_gas),
        )
        .change("timestamp", (old_value.timestamp, new_value.timestamp));
}
