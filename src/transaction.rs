use std::convert::TryFrom;
use std::error::Error;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::client::Account;
use crate::Record;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone)]
pub struct Transaction<S> {
    state: S,
}

impl Transaction<Recieved> {
    pub fn kind(&self) -> &TransactionKind {
        &self.state.kind
    }
}

impl Transaction<Processing> {
    fn new(kind: TransactionKind, ammount: Decimal) -> Self {
        Transaction {
            state: Processing::new(kind, ammount),
        }
    }

    pub fn process(self, account: &mut Account) -> Transaction<Completed> {
        match self.state.kind {
            TransactionKind::Deposit => account.deposit(self.state.ammount),
            TransactionKind::Withdrawal => account.withdraw(self.state.ammount),
            TransactionKind::Dispute => account.dispute(self.state.ammount),
            TransactionKind::Resolve => account.resolve(self.state.ammount),
            TransactionKind::Chargeback => account.chargeback(self.state.ammount),
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

    pub fn set_ammount(&mut self, ammount: Option<Decimal>) {
        self.state.ammount = ammount;
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

    pub fn set_ammount(&mut self, ammount: Option<Decimal>) {
        self.state.ammount = ammount;
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

    pub fn set_ammount(&mut self, ammount: Option<Decimal>) {
        self.state.ammount = ammount;
    }
}

/// Transaction always starts in this state.
pub struct Recieved {
    id: u32,
    kind: TransactionKind,
    ammount: Option<Decimal>,
}

/// Applies transaction to account.
#[derive(Debug, Clone)]
pub struct Processing {
    kind: TransactionKind,
    pub ammount: Decimal,
}

impl Processing {
    fn new(kind: TransactionKind, ammount: Decimal) -> Self {
        Processing { kind, ammount }
    }
}

/// Result of succecefully processing a deposit or withdrawal transaction.
#[derive(Debug, Clone)]
pub struct Completed;

/// Disputed transaction needs to be looked up for ammount of funds to hold.
#[derive(Debug, Clone)]
pub struct DisputeLookup {
    tx: u32,
    pub ammount: Option<Decimal>,
}

impl DisputeLookup {
    fn new(tx: u32) -> Self {
        DisputeLookup { tx, ammount: None }
    }
}

/// Dispute is resolved, held funds are released.
#[derive(Debug, Clone)]
pub struct Resolved {
    tx: u32,
    ammount: Option<Decimal>,
}

impl Resolved {
    fn new(tx: u32) -> Self {
        Resolved { tx, ammount: None }
    }
}

/// Dispute is charged back, held funds are withdrawn and their account locked.
#[derive(Debug, Clone)]
pub struct ChargedBack {
    tx: u32,
    ammount: Option<Decimal>,
}

impl ChargedBack {
    fn new(tx: u32) -> Self {
        ChargedBack { tx, ammount: None }
    }
}

impl From<Record> for Transaction<Recieved> {
    fn from(record: Record) -> Self {
        Transaction {
            state: Recieved {
                id: record.tx,
                kind: record.kind,
                ammount: record.ammount,
            },
        }
    }
}

#[derive(Debug)]
pub struct InvalidTransitionError;

impl std::fmt::Display for InvalidTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid state transition",)
    }
}

impl Error for InvalidTransitionError {}

impl TryFrom<Transaction<Recieved>> for Transaction<Processing> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Recieved>) -> Result<Self, Self::Error> {
        match prev.state.kind {
            TransactionKind::Deposit => {
                if let Some(ammount) = prev.state.ammount {
                    return Ok(Transaction::<Processing>::new(prev.state.kind, ammount));
                }
            }
            TransactionKind::Withdrawal => {
                if let Some(ammount) = prev.state.ammount {
                    return Ok(Transaction::<Processing>::new(prev.state.kind, ammount));
                }
            }
            _ => return Err(InvalidTransitionError),
        };

        Err(InvalidTransitionError)
    }
}

impl TryFrom<Transaction<Recieved>> for Transaction<DisputeLookup> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Recieved>) -> Result<Self, Self::Error> {
        match prev.state.kind {
            TransactionKind::Dispute => Ok(Transaction::<DisputeLookup>::new(prev.state.id)),
            _ => Err(InvalidTransitionError),
        }
    }
}

impl TryFrom<Transaction<DisputeLookup>> for Transaction<Processing> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<DisputeLookup>) -> Result<Self, Self::Error> {
        if let Some(ammount) = prev.state.ammount {
            return Ok(Transaction::<Processing>::new(
                TransactionKind::Dispute,
                ammount,
            ));
        }

        Err(InvalidTransitionError)
    }
}

impl TryFrom<Transaction<Recieved>> for Transaction<Resolved> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Recieved>) -> Result<Self, Self::Error> {
        match prev.state.kind {
            TransactionKind::Resolve => Ok(Transaction::<Resolved>::new(prev.state.id)),
            _ => Err(InvalidTransitionError),
        }
    }
}

impl TryFrom<Transaction<Resolved>> for Transaction<Processing> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Resolved>) -> Result<Self, Self::Error> {
        if let Some(ammount) = prev.state.ammount {
            return Ok(Transaction::<Processing>::new(
                TransactionKind::Resolve,
                ammount,
            ));
        }

        Err(InvalidTransitionError)
    }
}

impl TryFrom<Transaction<Recieved>> for Transaction<ChargedBack> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<Recieved>) -> Result<Self, Self::Error> {
        match prev.state.kind {
            TransactionKind::Chargeback => Ok(Transaction::<ChargedBack>::new(prev.state.id)),
            _ => Err(InvalidTransitionError),
        }
    }
}

impl TryFrom<Transaction<ChargedBack>> for Transaction<Processing> {
    type Error = InvalidTransitionError;

    fn try_from(prev: Transaction<ChargedBack>) -> Result<Self, Self::Error> {
        if let Some(ammount) = prev.state.ammount {
            return Ok(Transaction::<Processing>::new(
                TransactionKind::Chargeback,
                ammount,
            ));
        }

        Err(InvalidTransitionError)
    }
}
