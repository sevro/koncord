//! Client account management.
//!
//! This module provides the `Client` type which is the interface for working
//! with accounts and associating them with their Clients ID. It also
//! implements all operations on accounts.

use std::cmp::Ordering;

use rust_decimal::Decimal;
use serde::ser::{Serialize, SerializeStruct, Serializer};

/// The number of digits to the right of the decimal point.
///
/// A scale of four places past the decimal for all values.
const SCALE: u32 = 4;

/// A client represented by a Client ID and the associated account.
///
/// `Client` also implements `Serialize` directly to the output format.
#[derive(Debug, Eq, PartialEq)]
pub struct Client {
    id: u16,
    account: Account,
}

impl Client {
    /// Create a new `Client` with `id` and `0` balance.
    pub fn new(id: u16) -> Self {
        Client {
            id,
            account: Account::new(),
        }
    }

    /// Returns a mutable reference to the `Client`s `Account`.
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Returns a mutable reference to the `Client`s `Account`.
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

/// Client account.
///
/// Accounts have two primary states `Open` and `Frozen`. When accounts are
/// `Open` nearly all transactions are permitted with the exception of
/// withdrawals due to insufficient funds and any transaction with a negative
/// amount. All transactions are disallowed when the account is locked.
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

    /// Increase the available and total funds of the client account by amount.
    ///
    /// Only fails when the account is locked or amount is negative.
    pub(crate) fn deposit(&mut self, amount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => balance.deposit(amount),
            _ => (),
        }
    }

    /// Decrease the available and total funds of the client account by amount.
    ///
    /// Fails if account is locked, the account does not have sufficient
    /// available funds, or if the amount is negative.
    pub fn withdraw(&mut self, amount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => {
                balance.withdraw(amount);
            }
            _ => (),
        }
    }

    /// Associated funds moved to held.
    ///
    /// Available funds decreased by amount, held funds increased by amount,
    /// total funds remain the same. Fails if account is locked or amount is
    /// negative.
    pub fn dispute(&mut self, amount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => balance.dispute(amount),
            _ => (),
        }
    }

    /// Resolution to a dispute, releases held funds.
    ///
    /// Held funds decreased by amount, available funds increased by amount,
    /// total funds remain the same. Fails if account is locked or amount is
    /// negative.
    pub fn resolve(&mut self, amount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => balance.resolve(amount),
            _ => (),
        }
    }

    /// Final state of a dispute and represents the client reversing a transaction.
    ///
    /// Held funds and total funds are decreased by amount. Fails if account is
    /// locked or amount is negative.
    pub fn chargeback(&mut self, amount: Decimal) {
        match &mut self.inner {
            AccountInner::Open { balance } => {
                balance.chargeback(amount);
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
//
// Implements all balance manipulation operations.
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

    fn deposit(&mut self, amount: Decimal) {
        if amount > Decimal::ZERO {
            self.available += amount;
            self.total += amount;
        }
    }

    fn withdraw(&mut self, amount: Decimal) {
        if self.available > amount && amount > Decimal::ZERO {
            self.available -= amount;
            self.total -= amount;
        }
    }

    fn dispute(&mut self, amount: Decimal) {
        if amount > Decimal::ZERO {
            self.available -= amount;
            self.held += amount;
        }
    }

    fn resolve(&mut self, amount: Decimal) {
        if amount > Decimal::ZERO {
            self.available += amount;
            self.held -= amount;
        }
    }

    fn chargeback(&mut self, amount: Decimal) {
        if amount > Decimal::ZERO {
            self.held -= amount;
            self.total -= amount;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_new() {
        let zero = Decimal::ZERO;
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
    fn client_new_constraints() {
        let client = Client::new(u16::MIN);
        assert_eq!(
            client,
            Client {
                id: u16::MIN,
                account: Account::new(),
            }
        );

        let client = Client::new(u16::MAX);
        assert_eq!(
            client,
            Client {
                id: u16::MAX,
                account: Account::new(),
            }
        );
    }

    #[test]
    fn client_deposit() {
        let zero = Decimal::ZERO;
        let one_billion_dollars = Decimal::new(1_000_000_000, 0);
        let mut client = Client::new(42);
        client.get_mut().deposit(one_billion_dollars);

        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: one_billion_dollars,
                            held: zero,
                            total: one_billion_dollars
                        }
                    }
                },
            }
        );

        // Deposit should fail on locked account.
        //
        // We deposit one extra dollar to ensure we are not skipping all
        // transactions entirely and just checking `new()`. There is no way to
        // directly lock an account so we chargeback to lock it.
        let one = Decimal::ONE;
        let mut client = Client::new(1337);
        client.get_mut().deposit(one);
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().dispute(one_billion_dollars);
        client.get_mut().chargeback(one_billion_dollars);
        client.get_mut().deposit(one_billion_dollars);

        assert_eq!(
            client,
            Client {
                id: 1337,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Deposit should fail on negative amount.
        let mut client = Client::new(24);
        let negative_one = Decimal::NEGATIVE_ONE;
        client.get_mut().deposit(one);
        client.get_mut().deposit(negative_one);
        assert_eq!(
            client,
            Client {
                id: 24,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );
    }

    #[test]
    fn client_withdrawal() {
        let zero = Decimal::ZERO;
        let one = Decimal::ONE;
        let one_billion_dollars = Decimal::new(1_000_000_000, 0);
        let mut client = Client::new(42);

        client.get_mut().deposit(one);
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().withdraw(one_billion_dollars);

        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Withdrawal should fail on locked account.
        let leet = Decimal::new(1337, 0);
        let mut client = Client::new(1337);
        client.get_mut().deposit(leet);
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().dispute(one_billion_dollars);
        client.get_mut().chargeback(one_billion_dollars);
        client.get_mut().withdraw(one);

        assert_eq!(
            client,
            Client {
                id: 1337,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: leet,
                            held: zero,
                            total: leet,
                        }
                    }
                },
            }
        );

        // Withdrawal should fail on insufficient funds.
        let mut client = Client::new(0);
        client.get_mut().deposit(one);
        client.get_mut().withdraw(one_billion_dollars);
        assert_eq!(
            client,
            Client {
                id: 0,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Withdrawal should fail if amount is negative.
        let mut client = Client::new(7);
        client.get_mut().withdraw(Decimal::MIN);
        assert_eq!(
            client,
            Client {
                id: 7,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: zero,
                            held: zero,
                            total: zero,
                        }
                    }
                },
            }
        );

        // Withdrawal should fail on insufficient funds no matter how small.
        let mut client = Client::new(101);
        client.get_mut().withdraw(Decimal::new(1, SCALE));
        assert_eq!(
            client,
            Client {
                id: 101,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: zero,
                            held: zero,
                            total: zero,
                        }
                    }
                },
            }
        );
        let mut client = Client::new(102);
        client.get_mut().withdraw(Decimal::new(1, 28));
        assert_eq!(
            client,
            Client {
                id: 102,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: zero,
                            held: zero,
                            total: zero,
                        }
                    }
                },
            }
        );
    }

    #[test]
    fn client_dispute() {
        let zero = Decimal::ZERO;
        let one = Decimal::ONE;
        let negative_one = Decimal::NEGATIVE_ONE;
        let one_billion_dollars = Decimal::new(1_000_000_000, 0);
        let mut client = Client::new(42);
        client.get_mut().deposit(one);
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().dispute(one_billion_dollars);

        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: one,
                            held: one_billion_dollars,
                            total: one_billion_dollars + one,
                        }
                    }
                },
            }
        );

        // Dispute should fail on locked account.
        client.get_mut().chargeback(one_billion_dollars);
        client.get_mut().dispute(one_billion_dollars);
        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Dispute should fail on negative amount.
        let mut client = Client::new(24);
        client.get_mut().deposit(one);
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().dispute(negative_one);
        assert_eq!(
            client,
            Client {
                id: 24,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: one_billion_dollars + one,
                            held: zero,
                            total: one_billion_dollars + one,
                        }
                    }
                },
            }
        );
    }

    #[test]
    fn client_resolve() {
        let zero = Decimal::ZERO;
        let one = Decimal::ONE;
        let negative_one = Decimal::NEGATIVE_ONE;
        let one_billion_dollars = Decimal::new(1_000_000_000, 0);
        let mut client = Client::new(42);
        client.get_mut().deposit(one);
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().dispute(one_billion_dollars);
        client.get_mut().resolve(one_billion_dollars);

        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: one_billion_dollars + one,
                            held: zero,
                            total: one_billion_dollars + one,
                        }
                    }
                },
            }
        );

        // Dispute should fail on locked account.
        client.get_mut().dispute(one_billion_dollars);
        client.get_mut().chargeback(one_billion_dollars);
        client.get_mut().resolve(one_billion_dollars);
        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Dispute should fail on negative amount.
        let mut client = Client::new(24);
        client.get_mut().deposit(one);
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().dispute(one);
        client.get_mut().resolve(negative_one);
        assert_eq!(
            client,
            Client {
                id: 24,
                account: Account {
                    inner: AccountInner::Open {
                        balance: Balance {
                            available: one_billion_dollars,
                            held: one,
                            total: one_billion_dollars + one,
                        }
                    }
                },
            }
        );
    }

    #[test]
    fn client_chargeback() {
        let zero = Decimal::ZERO;
        let one = Decimal::ONE;
        let negative_one = Decimal::NEGATIVE_ONE;
        let one_billion_dollars = Decimal::new(1_000_000_000, 0);
        let mut client = Client::new(42);
        client.get_mut().deposit(one);
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().dispute(one_billion_dollars);
        client.get_mut().chargeback(one_billion_dollars);

        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Deposits should fail after chargeback.
        client.get_mut().deposit(one_billion_dollars);
        client.get_mut().deposit(one);
        client.get_mut().deposit(negative_one);
        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Withdrawals should fail after chargeback.
        client.get_mut().withdraw(one_billion_dollars);
        client.get_mut().withdraw(one);
        client.get_mut().withdraw(negative_one);
        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Disputes should fail after chargeback.
        client.get_mut().dispute(one_billion_dollars);
        client.get_mut().dispute(one);
        client.get_mut().dispute(negative_one);
        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Resolutions should fail after chargeback.
        client.get_mut().resolve(one_billion_dollars);
        client.get_mut().resolve(one);
        client.get_mut().resolve(negative_one);
        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );

        // Chargebacks should fail after chargeback.
        client.get_mut().chargeback(one_billion_dollars);
        client.get_mut().chargeback(one);
        client.get_mut().chargeback(negative_one);
        assert_eq!(
            client,
            Client {
                id: 42,
                account: Account {
                    inner: AccountInner::Frozen {
                        balance: Balance {
                            available: one,
                            held: zero,
                            total: one,
                        }
                    }
                },
            }
        );
    }

    #[test]
    fn account_inner_new() {
        let zero = Decimal::ZERO;
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
        let zero = Decimal::ZERO;
        let balance = Balance::new();
        assert_eq!(balance.available, zero);
        assert_eq!(balance.held, zero);
        assert_eq!(balance.total, zero);
    }
}
