use std::collections::HashMap;
use std::path::PathBuf;

use koncord;
use koncord::client::Client;

const BASE_EXPECTED: &str = "\
client,available,held,total,locked
1,1.5,0.0000,1.5,false
2,2,0.0000,2,false
";

const DISPUTE_EXPECTED: &str = "\
client,available,held,total,locked
1,1.5,1.5,3.0,false
2,2,0.0000,2,false
";

const RESOLVE_EXPECTED: &str = "\
client,available,held,total,locked
1,3.0,0.0,3.0,false
2,2,0.0000,2,false
";

const CHARGEBACK_EXPECTED: &str = "\
client,available,held,total,locked
1,1.5,0.0,1.5,true
2,2,0.0000,2,false
";

const TWENTY_EXPECTED: &str = "\
client,available,held,total,locked
1,20,0.0000,20,false
2,20,0.0000,20,false
";

#[test]
#[allow(unused_must_use)]
fn toy_deposit_withdraw() {
    let mut records_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    records_path.push("tests/data/toy/base.csv");

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
        BASE_EXPECTED
    );
}

#[test]
#[allow(unused_must_use)]
fn toy_dispute() {
    let mut records_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    records_path.push("tests/data/toy/dispute.csv");

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
        DISPUTE_EXPECTED
    );
}

#[test]
#[allow(unused_must_use)]
fn toy_resolve() {
    let mut records_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    records_path.push("tests/data/toy/resolve.csv");
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
        RESOLVE_EXPECTED
    );
}

#[test]
#[allow(unused_must_use)]
fn toy_chargeback() {
    let mut records_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    records_path.push("tests/data/toy/chargeback.csv");
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
        CHARGEBACK_EXPECTED
    );
}

#[test]
#[allow(unused_must_use)]
fn toy_twenty() {
    let mut records_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    records_path.push("tests/data/toy/twenty.csv");

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
        TWENTY_EXPECTED
    );
}
