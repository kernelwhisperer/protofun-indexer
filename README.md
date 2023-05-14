# Protofun sub-streams

## Example

```````any
```graphql
query LatestBaseFee {
  blocks(first: 1, orderBy: id, orderDirection: desc) {
    id
    BlockNum
    BaseFee
    BlockTime
  }
}
```

entity.id = block.number.toString()
entity.BaseFee = block.baseFeePerGas
entity.BlockNum = block.number
entity.BlockTime = block.timestamp

type Block @entity {
  id: ID!
  BlockNum: BigInt!
  BaseFee: BigInt
  BlockTime: BigInt!
} 

startBlock: 13800000
``````````

EIP 1559 Mainnet 12,965,000 August 5th, 2021
