# Koncord

Processes a CSV of transaction records and outputs the resulting client accounts to CSV.

## Transactions

Transaction flow is implemented as a state machine using `from` or `try_from`
as appropriate for transitions that do not require any additional context and
consuming methods where additional context is required. No stable and actively
maintained state machine libraries are available on [crates.io](crates.io) to
simplify the code here.

Each record is processed sequentially through the states shown below. The
dispute cache stores the dispute Transaction ID and amount before processing
to avoid unnecessary costly lookups for resolve and chargeback transactions.

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

Transactions are silently ignored during processing if:

* Transaction amount is negative.
* The account has ever had a chargeback and is therefor locked.
* Account has Insufficient funds for withdrawal.
* The referenced Transaction ID for a dispute, resolve, or chargeback does not exist.

The program will exit on errors including:

* Invalid records
* Attempting an invalid state transition

Transactions are implemented in [transactions.rs](src/transactions.rs) and rely
on the typesystem and the functional tests for correctness.

## Clients and Accounts

`Client`s and their accounts are implemented in [client.rs](src/clients.rs).
Account balances are represented using a fixed point datatype to avoid errors
introduced by floating point arithmetic. All operations on `Account` are
thoroughly unit tested, although it would be good to add fuzzing here.

## Dependencies

* [serde](https://crates.io/crates/serde)
* [csv](https://crates.io/crates/csv)
* [rust_decimal](https://crates.io/crates/rust_decimal)
