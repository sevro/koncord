use std::collections::HashMap;
use std::path::PathBuf;

use koncord;
use koncord::client::Client;

#[test]
#[allow(unused_must_use)]
fn clients_max() {
    let mut clients: HashMap<u16, Client> =
        HashMap::with_capacity(usize::try_from(u16::MAX).unwrap());

    let mut records = String::from("type,       client, tx, amount\n");
    for id in 0..=u16::MAX {
        let tx: u32 = id as u32 + 1;
        records += &format!("deposit,    {id},      {tx},  1.0\n");
    }

    let transaction_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(records.as_bytes()));

    let mut records_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    records_path.push("tests/data/toy/base.csv");

    koncord::run(
        &mut clients,
        transaction_records,
        records_path.to_str().unwrap(),
    );
}
