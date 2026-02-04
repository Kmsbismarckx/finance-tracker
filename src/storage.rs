use crate::models::Account;
use std::fs;
use std::path::Path;
use uuid::Uuid;

const STORAGE_FILE: &str = "accounts.json";

pub struct Storage {
    accounts: Vec<Account>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
        }
    }

    pub fn load() -> Result<Self, String> {
        if !Path::new(STORAGE_FILE).exists() {
            return Ok(Self::new());
        }

        let data =
            fs::read_to_string(STORAGE_FILE).map_err(|e| format!("Failed to read file: {}", e))?;

        let accounts: Vec<Account> =
            serde_json::from_str(&data).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        Ok(Self { accounts })
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.accounts)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(STORAGE_FILE, json).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    pub fn add_account(&mut self, account: Account) {
        self.accounts.push(account);
    }

    pub fn get_account(&self, id: &Uuid) -> Option<&Account> {
        self.accounts.iter().find(|a| &a.id == id)
    }

    pub fn get_all_accounts(&self) -> &[Account] {
        &self.accounts
    }

    pub fn get_account_mut(&mut self, id: &Uuid) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| &a.id == id)
    }

    pub fn delete_account(&mut self, id: &Uuid) -> Result<(), String> {
        let index = self
            .accounts
            .iter()
            .position(|a| &a.id == id)
            .ok_or("Account not found")?;

        self.accounts.remove(index);
        Ok(())
    }

    pub fn find_account_by_name(&self, name: &str) -> Option<&Account> {
        self.accounts
            .iter()
            .find(|a| a.name.to_lowercase() == name.to_lowercase())
    }
}
