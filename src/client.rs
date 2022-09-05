//! Clinet account management.
//!
//! This module provides the `Client` type which is the interface for working
//! with accounts.

use std::cmp::Ordering;

use rust_decimal::Decimal;
use serde::ser::{Serialize, SerializeStruct, Serializer};

/// The number of digits to the right of the decimal point.
///
/// A scale of four places past the decimal for all values.
const SCALE: u32 = 4;

#[derive(Debug, Eq, PartialEq)]
pub struct Client {
    id: u16,
    account: Account,
}

impl Client {
    pub fn new(id: u16) -> Self {
        Client {
            id,
            account: Account::new(),
        }
    }

    pub fn get_mut(&mut self) -> &mut Account {
        &mut self.account
    }
}

impl Ord for Client {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Client {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

// Required due to rust-csv issue "Support serializing of maps #98"
//
// See: https://github.com/BurntSushi/rust-csv/issues/98
impl Serialize for Client {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (locked, balance) = match &self.account.inner {
            AccountInner::Open { balance } => ("false", balance),
            AccountInner::Frozen { balance } => ("true", balance),
        };

        let mut row = serializer.serialize_struct("Client", 4)?;
        row.serialize_field("client", &self.id)?;
        row.serialize_field("available", &balance.available)?;
        row.serialize_field("held", &balance.held)?;
        row.serialize_field("total", &balance.total)?;
        row.serialize_field("locked", locked)?;
        row.end()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Account {
    inner: AccountInner,
}

impl Account {
    fn new() -> Self {
        Self {
            inner: AccountInner::new(),
        }
    }

    pub fn deposit(&mut self, ammount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => balance.deposit(ammount),
            _ => (),
        }
    }

    pub fn withdraw(&mut self, ammount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => {
                balance.withdraw(ammount);
            }
            _ => (),
        }
    }

    pub fn dispute(&mut self, ammount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => balance.dispute(ammount),
            _ => (),
        }
    }

    pub fn resolve(&mut self, ammount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => balance.resolve(ammount),
            _ => (),
        }
    }

    pub fn chargeback(&mut self, ammount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => {
                balance.chargeback(ammount);
                let balance = balance.clone();
                self.inner = AccountInner::Frozen { balance };
            }
            _ => (),
        }
    }
}

// Client account representation.
//
// Accounts only have two states `Open` where transactions are permitted and
// `Frozen` where all transactions are prohibited.
#[derive(Debug, Eq, PartialEq)]
enum AccountInner {
    Open { balance: Balance },
    Frozen { balance: Balance },
}

impl AccountInner {
    fn new() -> Self {
        Self::Open {
            balance: Balance::new(),
        }
    }
}

// Client account balance.
#[derive(Debug, Eq, PartialEq, Clone)]
struct Balance {
    available: Decimal,
    held: Decimal,
    total: Decimal,
}

impl Balance {
    fn new() -> Self {
        Balance {
            available: Decimal::new(0, SCALE),
            held: Decimal::new(0, SCALE),
            total: Decimal::new(0, SCALE),
        }
    }

    // Increase the available and total funds of the client account by ammount.
    //
    // Fails if account is locked.
    fn deposit(&mut self, ammount: Decimal) {
        self.available += ammount;
        self.total += ammount;
    }

    // Decrease the available and total funds of the client account by ammount.
    //
    // Fails if account is locked or the account does not have sufficient
    // available funds.
    fn withdraw(&mut self, ammount: Decimal) {
        if self.available > ammount {
            self.available -= ammount;
            self.total -= ammount;
        }
    }

    // Associated funds moved to held.
    //
    // Available funds decreased by ammount, held funds increased by ammount,
    // total funds remain the same.
    fn dispute(&mut self, ammount: Decimal) {
        self.available -= ammount;
        self.held += ammount;
    }

    // Resolution to a dispute, releases held funds.
    //
    // Held funds decreased by ammount, available funds increased by ammount,
    // total funds remain the same.
    fn resolve(&mut self, ammount: Decimal) {
        self.available += ammount;
        self.held -= ammount;
    }

    // Final state of a dispute and represents the client reversing a transaction.
    //
    // Held funds and total funds are decreased by ammount.
    fn chargeback(&mut self, ammount: Decimal) {
        self.held -= ammount;
        self.total -= ammount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_new() {
        let zero = Decimal::new(0, SCALE);
        let client = Client::new(42);

        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: zero,
                            held: zero,
                            total: zero
                        }
                    }
                },
            }
        );
    }

    #[test]
    fn account_inner_new() {
        let zero = Decimal::new(0, SCALE);
        let account = AccountInner::new();
        assert_eq!(
            account,
            AccountInner::Open {
                balance: Balance {
                    available: zero,
                    held: zero,
                    total: zero
                }
            }
        );
    }

    #[test]
    fn balance_new() {
        let zero = Decimal::new(0, SCALE);
        let balance = Balance::new();
        assert_eq!(balance.available, zero);
        assert_eq!(balance.held, zero);
        assert_eq!(balance.total, zero);
    }
}
