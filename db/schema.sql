create table block_meta
(
    id          integer not null constraint block_meta_pk primary key,
    hash        text,
    timestamp   text,
    gas_used integer,
    base_fee_per_gas integer
);

create table cursors
(
    id         text not null constraint cursor_pk primary key,
    cursor     text,
    block_num  bigint,
    block_id   text
);
