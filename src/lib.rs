mod pb;

use pb::sf::ethereum::block_meta::v1::BlockMeta;

use substreams::{scalar::BigInt, Hex};
use substreams_ethereum::pb::eth;

#[substreams::handlers::map]
fn map_block(block: eth::v2::Block) -> Result<BlockMeta, substreams::errors::Error> {
    let header = block.header.as_ref().unwrap();
    let base_fee_per_gas =
        BigInt::from_unsigned_bytes_be(&header.base_fee_per_gas.as_ref().unwrap().bytes)
            .to_string();

    Ok(BlockMeta {
        hash: Hex(&block.hash).to_string(),
        parent_hash: Hex(&header.parent_hash).to_string(),
        number: block.number,
        timestamp: header.timestamp.as_ref().unwrap().to_string(),
        gas_limit: header.gas_limit,
        gas_used: header.gas_used,
        base_fee_per_gas,
    })
}
