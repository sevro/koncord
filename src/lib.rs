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
    records_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut disputes: HashMap<u32, Decimal> = HashMap::new();

    for result in transaction_records.deserialize() {
        let record: Record = result?;
        let client: &mut Client = clients
            .entry(record.client_id())
            .or_insert(Client::new(record.client_id()));

        println!("{record:?}");
        process_record(record, client, &mut disputes, records_path)?;
    }

    Ok(())
}

// Process a single record.
fn process_record(
    record: Record,
    client: &mut Client,
    disputes: &mut HashMap<u32, Decimal>,
    records_path: &str,
) -> Result<(), Box<dyn Error>> {
    let recieved = Transaction::<Received>::from(record);

    match recieved.kind() {
        TransactionKind::Deposit | TransactionKind::Withdrawal => {
            let processing = Transaction::<Processing>::try_from(recieved)?;
            processing.process(client.get_mut());
        }
        TransactionKind::Dispute => {
            let mut dispute_lookup = Transaction::<DisputeLookup>::try_from(recieved)?;
            if let Some(record) = lookup_record(records_path, dispute_lookup.tx())? {
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
fn lookup_record(records_path: &str, tx: u32) -> Result<Option<Record>, Box<dyn Error>> {
    let mut search_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(records_path)?;

    let mut result: Option<Record> = None;
    for record_result in search_records.deserialize() {
        println!("{record_result:?}");
        let record: Record = record_result?;
        if record.tx() == tx {
            result = Some(record);
            break;
        }
    }

    Ok(result)
}
