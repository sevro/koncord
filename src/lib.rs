pub mod client;
mod transaction;

use std::collections::HashMap;
use std::error::Error;

use rust_decimal::Decimal;

use crate::client::Client;
use crate::transaction::{
    ChargedBack, DisputeLookup, Processing, Received, Record, Resolved, Transaction,
    TransactionKind,
};

/// Processes all transaction records.
///
/// Each record is processed sequentially through the states shown below. The
/// dispute cache stores the dispute Transaction ID and amount before
/// processing them to avoid unnecessary costly lookups for resolve and
/// chargeback transactions.
///
/// New clients are created with zero balances as new Client IDs are encountered.
///
/// ```diagram
///                    ┌──────┐
///      ┌───────────┬─┤Record├─┬──────────┐
///      │           │ └──────┘ │          │
///      │           │          │          │
/// ┌────▼─────┐ ┌───▼───┐ ┌────▼──┐ ┌─────▼────┐
/// │Deposit or│ │Dispute│ │Resolve│ │Chargeback│
/// │Withdrawal│ │Lookup │ │Lookup │ │Lookup    │
/// └────┬─────┘ └───┬───┘ └──▲─┬──┘ └─▲───┬────┘
///      │           │        │ │      │   │
///      │    ┌──────┴──────┐ │ │      │   │
///      │    │Dispute Cache├─┴─┼──────┘   │
///      │    └──────┬──────┘   │          │
///      │           │          │          │
///      │           │          │          │
/// ┌────▼─────┐     │          │          │
/// │Processing◄─────┴──────────┴──────────┘
/// └────┬─────┘
///      │
///      │
/// ┌────┴───┐
/// │Complete│
/// └────────┘
/// ```
pub fn run<R: std::io::Read + std::io::Seek>(
    clients: &mut HashMap<u16, Client>,
    mut transaction_records: csv::Reader<R>,
    mut search_records: csv::Reader<R>,
) -> Result<(), Box<dyn Error>> {
    let mut disputes: HashMap<u32, Decimal> = HashMap::new();

    for result in transaction_records.deserialize() {
        let record: Record = result?;
        let client: &mut Client = clients
            .entry(record.client_id())
            .or_insert(Client::new(record.client_id()));

        process_record(record, client, &mut disputes, &mut search_records)?;
    }

    Ok(())
}

// Process a single record.
fn process_record<R: std::io::Read + std::io::Seek>(
    record: Record,
    client: &mut Client,
    disputes: &mut HashMap<u32, Decimal>,
    search_records: &mut csv::Reader<R>,
) -> Result<(), Box<dyn Error>> {
    let recieved = Transaction::<Received>::from(record);

    match recieved.kind() {
        TransactionKind::Deposit | TransactionKind::Withdrawal => {
            let processing = Transaction::<Processing>::try_from(recieved)?;
            processing.process(client.get_mut());
        }
        TransactionKind::Dispute => {
            let mut dispute_lookup = Transaction::<DisputeLookup>::try_from(recieved)?;
            if let Some(record) = lookup_record(search_records, dispute_lookup.tx())? {
                disputes.insert(record.tx(), record.amount().unwrap());
                dispute_lookup.set_amount(record.amount());
                let processing = Transaction::<Processing>::try_from(dispute_lookup)?;
                processing.process(client.get_mut());
            }
        }
        TransactionKind::Resolve => {
            let mut resolved = Transaction::<Resolved>::try_from(recieved)?;
            if let Some(amount) = disputes.remove(&resolved.tx()) {
                resolved.set_amount(Some(amount));
                let processing = Transaction::<Processing>::try_from(resolved)?;
                processing.process(client.get_mut());
            }
        }
        TransactionKind::Chargeback => {
            let mut chargeback = Transaction::<ChargedBack>::try_from(recieved)?;
            if let Some(amount) = disputes.remove(&chargeback.tx()) {
                chargeback.set_amount(Some(amount));
                let processing = Transaction::<Processing>::try_from(chargeback)?;
                processing.process(client.get_mut());
            }
        }
    }

    Ok(())
}

// Return record matching Transaction ID `tx` if found, else None.
fn lookup_record<R: std::io::Read + std::io::Seek>(
    search_records: &mut csv::Reader<R>,
    tx: u32,
) -> Result<Option<Record>, Box<dyn Error>> {
    let mut result: Option<Record> = None;
    for record_result in search_records.deserialize() {
        let record: Record = record_result?;
        if record.tx() == tx {
            result = Some(record);
            break;
        }
    }

    search_records.seek(csv::Position::new())?;
    Ok(result)
}
