pub mod client;

mod transaction;

use std::collections::HashMap;
use std::error::Error;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::client::Client;
use crate::transaction::*;

/// Transaction record.
///
/// A single transaction record.
#[derive(Debug, Deserialize)]
struct Record {
    /// Transaction Type.
    #[serde(rename = "type")]
    kind: TransactionKind,
    /// Client ID.
    client: u16,
    /// Transaction ID.
    tx: u32,
    /// Transaction Ammount.
    ///
    /// A decimal value with a precision of up to four places past the decimal.
    ammount: Option<Decimal>,
}

impl Record {
    pub fn client_id(&self) -> u16 {
        self.client
    }
}

// Process a single record.
fn process_record<R: std::io::Read + std::io::Seek>(
    record: Record,
    client: &mut Client,
    disputes: &mut HashMap<u32, Decimal>,
    search_records: &mut csv::Reader<R>,
) -> Result<(), Box<dyn Error>> {
    let recieved = Transaction::<Recieved>::from(record);

    match recieved.kind() {
        TransactionKind::Deposit | TransactionKind::Withdrawal => {
            let processing = Transaction::<Processing>::try_from(recieved)?;
            processing.process(&mut client.account);
        }
        TransactionKind::Dispute => {
            let mut dispute_lookup = Transaction::<DisputeLookup>::try_from(recieved)?;
            if let Some(record) = lookup_record(search_records, dispute_lookup.tx())? {
                disputes.insert(record.tx, record.ammount.unwrap());
                dispute_lookup.set_ammount(record.ammount);
                let processing = Transaction::<Processing>::try_from(dispute_lookup)?;
                processing.process(&mut client.account);
            }
        }
        TransactionKind::Resolve => {
            let mut resolved = Transaction::<Resolved>::try_from(recieved)?;
            if let Some(ammount) = disputes.remove(&resolved.tx()) {
                resolved.set_ammount(Some(ammount));
                let processing = Transaction::<Processing>::try_from(resolved)?;
                processing.process(&mut client.account);
            }
        }
        TransactionKind::Chargeback => {
            let mut chargeback = Transaction::<ChargedBack>::try_from(recieved)?;
            if let Some(ammount) = disputes.remove(&chargeback.tx()) {
                chargeback.set_ammount(Some(ammount));
                let processing = Transaction::<Processing>::try_from(chargeback)?;
                processing.process(&mut client.account);
            }
        }
    }

    Ok(())
}

// Return record matching Transaction ID if found, else None.
//
// Looks from current record position backwards and returns to start pos when
// finished.
//
// Failure leaves the records in a bad state.
fn lookup_record<R: std::io::Read + std::io::Seek>(
    search_records: &mut csv::Reader<R>,
    tx: u32,
) -> Result<Option<Record>, Box<dyn Error>> {
    let mut result: Option<Record> = None;
    for record_result in search_records.deserialize() {
        let record: Record = record_result?;
        if record.tx == tx {
            result = Some(record);
            break;
        }
    }

    search_records.seek(csv::Position::new())?;
    Ok(result)
}

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
