use std::collections::HashMap;
use std::path::PathBuf;

use koncord;
use koncord::client::Client;

const COMPLEX_EXPECTED: &str = "\
client,available,held,total,locked
1,1.0,0.5,1.5,false
2,1.5,0.5,2,false
999,0.0000,0.0000,0.0000,false
1000,500,0.0000,500,false
1001,0.0000,0.0000,0.0000,false
";

#[test]
#[allow(unused_must_use)]
fn complex() {
    let mut records_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    records_path.push("tests/data/complex.csv");

    let mut clients: HashMap<u16, Client> =
        HashMap::with_capacity(usize::try_from(u16::MAX).unwrap());

    let transaction_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(&records_path)
        .unwrap();

    koncord::run(
        &mut clients,
        transaction_records,
        records_path.to_str().unwrap(),
    );

    let mut wtr = csv::Writer::from_writer(vec![]);
    let mut clients: Vec<&Client> = clients.values().collect();
    clients.sort();
    for client in clients {
        wtr.serialize(client);
    }
    wtr.flush();
    assert_eq!(
        String::from_utf8(wtr.into_inner().unwrap()).unwrap(),
        COMPLEX_EXPECTED
    );
}
