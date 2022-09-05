use std::collections::HashMap;
use std::path::PathBuf;

use koncord;
use koncord::client::Client;

#[test]
#[ignore]
#[allow(unused_must_use)]
fn stress_100k_transactions() {
    let mut stress_test = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    stress_test.push("tests/data/100k_transactions.csv");

    let mut clients: HashMap<u16, Client> =
        HashMap::with_capacity(usize::try_from(u16::MAX).unwrap());

    let transaction_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(&stress_test)
        .unwrap();

    koncord::run(
        &mut clients,
        transaction_records,
        stress_test.to_str().unwrap(),
    )
    .unwrap();
}
