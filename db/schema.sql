create table block_meta
(
    id          text not null constraint block_meta_pk primary key,
    at          text,
    hash        text,
    number      integer,
    timestamp   text,
    gas_used integer,
    base_fee_per_gas text
);

create table cursors
(
    id         text not null constraint cursor_pk primary key,
    cursor     text,
    block_num  bigint,
    block_id   text
);
