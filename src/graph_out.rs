use substreams::store::{self, DeltaProto};
use substreams_entity_change::tables::Tables;
mod pb;

use crate::pb::sf::ethereum::block_meta::v1::BlockMeta;

pub fn block_meta_to_entities_changes(
    tables: &mut Tables,
    deltas: store::Deltas<DeltaProto<BlockMeta>>,
) {
    use substreams::pb::substreams::store_delta::Operation;

    for delta in deltas.deltas {
        match delta.operation {
            Operation::Create => push_create(tables, &delta.key,  delta.new_value),
            Operation::Update => push_update(
                tables,
                &delta.key,
                delta.new_value,
            ),
            Operation::Delete => todo!(),
            x => panic!("unsupported operation {:?}", x),
        }
    }
}

fn push_create(tables: &mut Tables, key: &str, value: BlockMeta) {
    tables
        .create_row("block_meta", key)
        .set("number", value.number)
        .set("hash", value.hash)
        .set("gas_used", value.gas_used)
        .set("base_fee_per_gas", value.base_fee_per_gas)
        .set("timestamp", value.timestamp);
}

fn push_update(
    tables: &mut Tables,
    key: &str,
    value: BlockMeta,
) {
    tables
        .update_row("block_meta", key)
        .set("number", value.number)
        .set("hash", value.hash)
        .set("gas_used", value.gas_used)
        .set("base_fee_per_gas", value.base_fee_per_gas)
        .set("timestamp", value.timestamp);
}
