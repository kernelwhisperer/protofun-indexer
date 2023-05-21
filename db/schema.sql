create table blocks
(
    id                  integer not null constraint block_meta_pk primary key,
    hash                text,
    timestamp           text,
    gas_used            integer,
    base_fee_per_gas    text,
    min_gas_price       text,
    max_gas_price       text,
    gas_fees            text,
    burned_fees         text,
    miner_tips          text
);

create table transactions
(
    id                       text not null constraint transaction_meta_pk primary key,
    block_number             integer,
    hash                     text,
    gas_used                 integer,
    gas_price                text,
    gas_fee                  text,
    txn_type                 integer,
    max_priority_fee_per_gas text,
    burned_fee               text,
    miner_tip                text

    -- foreign key (block_number) references blocks(id)
);

create table cursors
(
    id         text not null constraint cursor_pk primary key,
    cursor     text,
    block_num  bigint,
    block_id   text
);
