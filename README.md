# Koncord

Processes a CSV of transaction records and outputs the resulting client accounts to CSV.

## Usage

```
cargo run -- transactions.csv > accounts.csv
```

## Transactions

Transaction flow is implemented as a state machine using `from` or `try_from`
as appropriate for transitions that do not require any additional context and
consuming methods where additional context is required. No stable and actively
maintained state machine libraries are available on [crates.io](https://crates.io)
to simplify the code here.

Each record is processed sequentially through the states shown below. The
dispute cache stores the dispute Transaction ID and amount before processing
to avoid unnecessary costly lookups for resolve and chargeback transactions.
Due to a bug found running the [100k_transacitons.csv](tests/data/100k_transactions.csv)
test dispute lookups always creates a new reader to avoid
[`seek`](https://docs.rs/csv/latest/csv/struct.Reader.html#method.seek)
sometimes attempting to process the headers as a record. Parallelizing dispute,
resolve, and chargeback would be the highest impact optimization.

```
                   ┌──────┐
     ┌───────────┬─┤Record├─┬──────────┐
     │           │ └──────┘ │          │
     │           │          │          │
┌────▼─────┐ ┌───▼───┐ ┌────▼──┐ ┌─────▼────┐
│Deposit or│ │Dispute│ │Resolve│ │Chargeback│
│Withdrawal│ │Lookup │ │Lookup │ │Lookup    │
└────┬─────┘ └───┬───┘ └──▲─┬──┘ └─▲───┬────┘
     │           │        │ │      │   │
     │    ┌──────┴──────┐ │ │      │   │
     │    │Dispute Cache├─┴─┼──────┘   │
     │    └──────┬──────┘   │          │
     │           │          │          │
     │           │          │          │
┌────▼─────┐     │          │          │
│Processing◄─────┴──────────┴──────────┘
└────┬─────┘
     │
     │
┌────┴───┐
│Complete│
└────────┘
```

Many transactions may be silently ignored, this would be a good place to add
error handling and at least logging. Transactions that are silently ignored
include:

* Transaction amount is negative.
* The account has ever had a chargeback and is therefor locked.
* Account has Insufficient funds for withdrawal.
* The referenced Transaction ID for a dispute, resolve, or chargeback does not exist.
* Dispute, resolve, and chargebacks against accounts you don't own are also ignored.

The program will exit on errors including:

* Invalid records
* Attempting an invalid state transition

Transactions are implemented in [transaction.rs](src/transaction.rs) and rely
on the typesystem and the functional tests for correctness.

## Clients and Accounts

`Client`s and their accounts are implemented in [client.rs](src/client.rs).
Account balances are represented using a fixed point datatype to avoid errors
introduced by floating point arithmetic. All operations on `Account` are
thoroughly unit tested, although it would be good to add fuzzing here.

## Functional Tests

* [Basic functionality](tests/toys.rs)
* [Complex functionality](tests/complex.rs)
* [Maximum number of clients](tests/clients_max.rs)
* [100K transactions](tests/tx_stress.rs): Ignored by default due to time required.

Data files can be found in [tests/data/](tests/data/).

## Dependencies

* [serde](https://crates.io/crates/serde)
* [csv](https://crates.io/crates/csv)
* [rust_decimal](https://crates.io/crates/rust_decimal)
