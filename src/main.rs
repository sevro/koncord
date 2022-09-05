use std::collections::HashMap;
use std::error::Error;

use koncord::client::Client;
use koncord::run;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut clients: HashMap<u16, Client> = HashMap::with_capacity(usize::try_from(u16::MAX)?);
    let records_path = &args[1];

    let transaction_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(records_path)?;

    run(&mut clients, transaction_records, records_path)?;

    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for client in clients.values() {
        wtr.serialize(client)?;
    }
    wtr.flush()?;

    Ok(())
}
