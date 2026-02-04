use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub balance: f64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
}

impl Account {
    pub fn new(name: String, currency: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            balance: 0.0,
            currency,
            created_at: Utc::now(),
        }
    }

    pub fn deposit(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }
        self.balance += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }
        if self.balance < amount {
            return Err("Insufficient funds".to_string());
        }
        self.balance -= amount;
        Ok(())
    }
}
