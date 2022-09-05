use std::collections::HashMap;

use koncord;
use koncord::client::Client;

const BASE_DATA: &str = "\
type,       client, tx, amount
deposit,    1,      1,  1.0
deposit,    2,      2,  2.0
deposit,    1,      3,  2.0
withdrawal, 1,      4,  1.5
withdrawal, 2,      5,  3.0";

const BASE_EXPECTED: &str = "\
client,available,held,total,locked
1,1.5,0.0000,1.5,false
2,2,0.0000,2,false
";

const DISPUTE_DATA: &str = "
deposit,    1,      6,  1.5
dispute,    1,      6
";

const DISPUTE_EXPECTED: &str = "\
client,available,held,total,locked
1,1.5,1.5,3.0,false
2,2,0.0000,2,false
";

const RESOLVE_DATA: &str = "
deposit,    1,      6,  1.5
dispute,    1,      6
resolve,    1,      6
";

const RESOLVE_EXPECTED: &str = "\
client,available,held,total,locked
1,3.0,0.0,3.0,false
2,2,0.0000,2,false
";

const CHARGEBACK_DATA: &str = "
deposit,    1,      6,  1.5
dispute,    1,      6
chargeback, 1,      6
";

const CHARGEBACK_EXPECTED: &str = "\
client,available,held,total,locked
1,1.5,0.0,1.5,true
2,2,0.0000,2,false
";

#[test]
#[allow(unused_must_use)]
fn toy_deposit_withdraw() {
    let mut clients: HashMap<u16, Client> =
        HashMap::with_capacity(usize::try_from(u16::MAX).unwrap());

    let transaction_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(BASE_DATA.as_bytes()));

    let search_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(BASE_DATA.as_bytes()));

    koncord::run(&mut clients, transaction_records, search_records);

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
    let mut clients: HashMap<u16, Client> =
        HashMap::with_capacity(usize::try_from(u16::MAX).unwrap());

    let test_dispute_data = String::from(BASE_DATA) + DISPUTE_DATA;

    let transaction_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(test_dispute_data.as_bytes()));

    let search_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(test_dispute_data.as_bytes()));

    koncord::run(&mut clients, transaction_records, search_records);

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
    let mut clients: HashMap<u16, Client> =
        HashMap::with_capacity(usize::try_from(u16::MAX).unwrap());

    let test_resolve_data = String::from(BASE_DATA) + RESOLVE_DATA;

    let transaction_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(test_resolve_data.as_bytes()));

    let search_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(test_resolve_data.as_bytes()));

    koncord::run(&mut clients, transaction_records, search_records);

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
    let mut clients: HashMap<u16, Client> =
        HashMap::with_capacity(usize::try_from(u16::MAX).unwrap());

    let test_chargeback_data = String::from(BASE_DATA) + CHARGEBACK_DATA;

    let transaction_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(test_chargeback_data.as_bytes()));

    let search_records = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(std::io::Cursor::new(test_chargeback_data.as_bytes()));

    koncord::run(&mut clients, transaction_records, search_records);

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
