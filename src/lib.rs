mod pb;

use pb::sf::ethereum::block_meta::v1::BlockMeta;

use substreams::{scalar::BigInt, Hex};
use substreams_ethereum::pb::eth;

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
    let base_fee_per_gas = header.base_fee_per_gas.as_ref().map_or("N/A".to_string(), |base_fee| {
        BigInt::from_unsigned_bytes_be(&base_fee.bytes).to_string()
    });

    BlockMeta {
        hash: Hex(&block.hash).to_string(),
        number: block.number,
        timestamp: header.timestamp.as_ref().unwrap().to_string(),
        gas_used: header.gas_used,
        base_fee_per_gas,
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
pub fn db_out(
    block_meta: store::Deltas<DeltaProto<BlockMeta>>
) -> Result<DatabaseChanges, Error> {
    let mut database_changes: DatabaseChanges = Default::default();
    db_out::block_meta_to_database_changes(&mut database_changes, block_meta);

    Ok(database_changes)
}

#[substreams::handlers::map]
pub fn graph_out(
    block_meta: store::Deltas<DeltaProto<BlockMeta>>
) -> Result<EntityChanges, Error> {
    let mut entity_changes: EntityChanges = Default::default();
    graph_out::block_meta_to_entities_changes(&mut entity_changes, block_meta);

    Ok(entity_changes)
}
