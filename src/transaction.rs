//! Transactions provides a state machine to process records to completion.
use std::convert::TryFrom;
use std::error::Error;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::client::Account;

/// Transaction record.
///
/// The representation of a raw transaction record as received by the service.
#[derive(Debug, Deserialize)]
pub struct Record {
    /// Transaction Type.
    #[serde(rename = "type")]
    kind: TransactionKind,
    /// Client ID.
    client: u16,
    /// Transaction ID.
    tx: u32,
    /// Transaction amount.
    ///
    /// A decimal value with a precision of up to four places past the decimal.
    amount: Option<Decimal>,
}

impl Record {
    pub fn client_id(&self) -> u16 {
        self.client
    }

    pub fn tx(&self) -> u32 {
        self.tx
    }

    pub fn amount(&self) -> Option<Decimal> {
        self.amount
    }
}

/// Types of transactions.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// A single transaction.
///
/// Implements a state machine for transactions.
#[derive(Debug, Clone)]
pub struct Transaction<S> {
    state: S,
}

impl Transaction<Received> {
    pub fn kind(&self) -> &TransactionKind {
        &self.state.kind
    }
}

impl Transaction<Processing> {
    fn new(kind: TransactionKind, amount: Decimal) -> Self {
        Transaction {
            state: Processing::new(kind, amount),
        }
    }

    pub fn process(self, account: &mut Account) -> Transaction<Completed> {
        match self.state.kind {
            TransactionKind::Deposit => account.deposit(self.state.amount),
            TransactionKind::Withdrawal => account.withdraw(self.state.amount),
            TransactionKind::Dispute => account.dispute(self.state.amount),
            TransactionKind::Resolve => account.resolve(self.state.amount),
            TransactionKind::Chargeback => account.chargeback(self.state.amount),
        }

        Transaction::<Completed>::new()
    }
}

impl Transaction<Completed> {
    fn new() -> Self {
        Transaction { state: Completed }
    }
}

impl Transaction<DisputeLookup> {
    fn new(tx: u32) -> Self {
        Transaction {
            state: DisputeLookup::new(tx),
        }
    }

    pub fn tx(&self) -> u32 {
        self.state.tx
    }

    pub fn set_amount(&mut self, amount: Option<Decimal>) {
        self.state.amount = amount;
    }
}

impl Transaction<Resolved> {
    fn new(tx: u32) -> Self {
        Transaction {
            state: Resolved::new(tx),
        }
    }

    pub fn tx(&self) -> u32 {
        self.state.tx
    }

    pub fn set_amount(&mut self, amount: Option<Decimal>) {
        self.state.amount = amount;
    }
}

impl Transaction<ChargedBack> {
    fn new(tx: u32) -> Self {
        Transaction {
            state: ChargedBack::new(tx),
        }
    }

    pub fn tx(&self) -> u32 {
        self.state.tx
    }

    pub fn set_amount(&mut self, amount: Option<Decimal>) {
        self.state.amount = amount;
    }
}

/// Transaction always starts in this state.
#[derive(Debug, Clone)]
pub struct Received {
    id: u32,
    kind: TransactionKind,
    amount: Option<Decimal>,
}

/// Applies transaction to account.
#[derive(Debug, Clone)]
pub struct Processing {
    kind: TransactionKind,
    pub amount: Decimal,
}

impl Processing {
    fn new(kind: TransactionKind, amount: Decimal) -> Self {
        Processing { kind, amount }
    }
}

/// Result of succecefully processing a deposit or withdrawal transaction.
#[derive(Debug, Clone)]
pub struct Completed;

/// Disputed transaction needs to be looked up for amount of funds to hold.
#[derive(Debug, Clone)]
pub struct DisputeLookup {
    tx: u32,
    pub amount: Option<Decimal>,
}

impl DisputeLookup {
    fn new(tx: u32) -> Self {
        DisputeLookup { tx, amount: None }
    }
}

/// Dispute is resolved, held funds are released.
#[derive(Debug, Clone)]
pub struct Resolved {
    tx: u32,
    amount: Option<Decimal>,
}

impl Resolved {
    fn new(tx: u32) -> Self {
        Resolved { tx, amount: None }
    }
}

/// Dispute is charged back, held funds are withdrawn and their account locked.
#[derive(Debug, Clone)]
pub struct ChargedBack {
    tx: u32,
    amount: Option<Decimal>,
}

impl ChargedBack {
    fn new(tx: u32) -> Self {
        ChargedBack { tx, amount: None }
    }
}

impl From<Record> for Transaction<Received> {
    fn from(record: Record) -> Self {
        Transaction {
            state: Received {
                id: record.tx,
                kind: record.kind,
                amount: record.amount,
            },
        }
    }
}

#[derive(Debug)]
pub struct InvalidTransitionError {
    from: String,
    to: String,
}

impl std::fmt::Display for InvalidTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid state transition from {} to {}",
            self.from, self.to
        )
    }
}

impl Error for InvalidTransitionError {}

impl TryFrom<Transaction<Received>> for Transaction<Processing> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Received>) -> Result<Self, Self::Error> {
        println!("previous transaction: {prev:?}");
        match prev.state.kind {
            TransactionKind::Deposit => {
                if let Some(amount) = prev.state.amount {
                    return Ok(Transaction::<Processing>::new(prev.state.kind, amount));
                }
            }
            TransactionKind::Withdrawal => {
                if let Some(amount) = prev.state.amount {
                    return Ok(Transaction::<Processing>::new(prev.state.kind, amount));
                }
            }
            kind => {
                return Err(InvalidTransitionError {
                    from: "Transaction<Received>".to_string(),
                    to: format!("{kind:?}"),
                })
            }
        };

        Err(InvalidTransitionError {
            from: "Transaction<Received>".to_string(),
            to: "Transaction<Processing>".to_string(),
        })
    }
}

impl TryFrom<Transaction<Received>> for Transaction<DisputeLookup> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Received>) -> Result<Self, Self::Error> {
        match prev.state.kind {
            TransactionKind::Dispute => Ok(Transaction::<DisputeLookup>::new(prev.state.id)),
            kind => {
                return Err(InvalidTransitionError {
                    from: "Transaction<Received>".to_string(),
                    to: format!("{kind:?}"),
                })
            }
        }
    }
}

impl TryFrom<Transaction<DisputeLookup>> for Transaction<Processing> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<DisputeLookup>) -> Result<Self, Self::Error> {
        if let Some(amount) = prev.state.amount {
            return Ok(Transaction::<Processing>::new(
                TransactionKind::Dispute,
                amount,
            ));
        }

        Err(InvalidTransitionError {
            from: "Transaction<DisputeLookup>".to_string(),
            to: "Transaction<Processing>".to_string(),
        })
    }
}

impl TryFrom<Transaction<Received>> for Transaction<Resolved> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Received>) -> Result<Self, Self::Error> {
        match prev.state.kind {
            TransactionKind::Resolve => Ok(Transaction::<Resolved>::new(prev.state.id)),
            kind => {
                return Err(InvalidTransitionError {
                    from: "Transaction<Received>".to_string(),
                    to: format!("{kind:?}"),
                })
            }
        }
    }
}

impl TryFrom<Transaction<Resolved>> for Transaction<Processing> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Resolved>) -> Result<Self, Self::Error> {
        if let Some(amount) = prev.state.amount {
            return Ok(Transaction::<Processing>::new(
                TransactionKind::Resolve,
                amount,
            ));
        }

        Err(InvalidTransitionError {
            from: "Transaction<Resolved>".to_string(),
            to: "Transaction<Processing>".to_string(),
        })
    }
}

impl TryFrom<Transaction<Received>> for Transaction<ChargedBack> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Received>) -> Result<Self, Self::Error> {
        match prev.state.kind {
            TransactionKind::Chargeback => Ok(Transaction::<ChargedBack>::new(prev.state.id)),
            kind => {
                return Err(InvalidTransitionError {
                    from: "Transaction<Received>".to_string(),
                    to: format!("{kind:?}"),
                })
            }
        }
    }
}

impl TryFrom<Transaction<ChargedBack>> for Transaction<Processing> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<ChargedBack>) -> Result<Self, Self::Error> {
        if let Some(amount) = prev.state.amount {
            return Ok(Transaction::<Processing>::new(
                TransactionKind::Chargeback,
                amount,
            ));
        }

        Err(InvalidTransitionError {
            from: "Transaction<ChargedBack>".to_string(),
            to: "Transaction<Processing>".to_string(),
        })
    }
}
