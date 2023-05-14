mod pb;

use pb::sf::ethereum::block_meta::v1::BlockMeta;

use substreams::{scalar::BigInt, Hex};
use substreams_ethereum::pb::eth;

#[path = "db_out.rs"]
mod db;

use substreams::errors::Error;
use substreams::store::{DeltaProto, StoreSetIfNotExistsProto};
use substreams::{prelude::*, store};
use substreams_database_change::pb::database::DatabaseChanges;


#[substreams::handlers::map]
fn map_block(block: eth::v2::Block) -> Result<BlockMeta, substreams::errors::Error> {
    let header = block.header.as_ref().unwrap();
    let base_fee_per_gas =
        BigInt::from_unsigned_bytes_be(&header.base_fee_per_gas.as_ref().unwrap().bytes)
            .to_string();

    Ok(BlockMeta {
        hash: Hex(&block.hash).to_string(),
        number: block.number,
        timestamp: header.timestamp.as_ref().unwrap().to_string(),
        gas_used: header.gas_used,
        base_fee_per_gas,
    })
}

fn map_block_to_meta(block: eth::v2::Block) -> BlockMeta {
    let header = block.header.as_ref().unwrap();
    let base_fee_per_gas =
        BigInt::from_unsigned_bytes_be(&header.base_fee_per_gas.as_ref().unwrap().bytes)
            .to_string();

    BlockMeta {
        hash: Hex(&block.hash).to_string(),
        number: block.number,
        timestamp: header.timestamp.as_ref().unwrap().to_string(),
        gas_used: header.gas_used,
        base_fee_per_gas,
    }
}

#[substreams::handlers::map]
pub fn db_out(
    block_meta_start: store::Deltas<DeltaProto<BlockMeta>>
) -> Result<DatabaseChanges, Error> {
    let mut database_changes: DatabaseChanges = Default::default();
    db::block_meta_to_database_changes(&mut database_changes, block_meta_start);

    Ok(database_changes)
}

#[substreams::handlers::store]
fn store_block_meta_start(block: eth::v2::Block, store: StoreSetIfNotExistsProto<BlockMeta>) {
    let meta: BlockMeta = map_block_to_meta(block);

    store.set_if_not_exists(0, &meta.hash, &meta);
}
