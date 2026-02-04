mod models;
mod storage;

use clap::{Parser, Subcommand};
use models::Account;
use storage::Storage;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "finance-tracker")]
#[command(about = "A personal finance tracker", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new account
    AddAccount {
        /// Name of the account
        name: String,
        /// Currency code (e.g., USD, EUR, RUB)
        #[arg(default_value = "USD")]
        currency: String,
    },
    /// List all accounts
    ListAccounts,
    /// Delete an account
    DeleteAccount {
        /// Account ID or name
        account: String,
    },
    /// Deposit money to an account
    Deposit {
        /// Account ID or name
        account: String,
        /// Amount to deposit
        amount: f64,
    },
    /// Withdraw money from an account
    Withdraw {
        /// Account ID or name
        account: String,
        /// Amount to withdraw
        amount: f64,
    },
    /// Show account details
    ShowAccount {
        /// Account ID or name
        account: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::AddAccount { name, currency } => add_account(name, currency),
        Commands::ListAccounts => list_accounts(),
        Commands::DeleteAccount { account } => delete_account(account),
        Commands::Deposit { account, amount } => deposit(account, amount),
        Commands::Withdraw { account, amount } => withdraw(account, amount),
        Commands::ShowAccount { account } => show_account(account),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn add_account(name: String, currency: String) -> Result<(), String> {
    let mut storage = Storage::load()?;

    if storage.find_account_by_name(&name).is_some() {
        return Err(format!("Account with name '{}' already exists", name));
    }

    let account = Account::new(name.clone(), currency);
    println!("Created account: {} (ID: {})", name, account.id);

    storage.add_account(account);
    storage.save()?;

    Ok(())
}

fn list_accounts() -> Result<(), String> {
    let storage = Storage::load()?;
    let accounts = storage.get_all_accounts();

    if accounts.is_empty() {
        println!("No accounts found.");
        return Ok(());
    }

    println!(
        "\n{:<38} {:<20} {:<15} {:<10}",
        "ID", "Name", "Balance", "Currency"
    );
    println!("{}", "-".repeat(85));

    for account in accounts {
        println!(
            "{:<38} {:<20} {:>15.2} {:<10}",
            account.id, account.name, account.balance, account.currency
        );
    }

    Ok(())
}

fn delete_account(account_identifier: String) -> Result<(), String> {
    let mut storage = Storage::load()?;

    let account_id = if let Ok(uuid) = Uuid::parse_str(&account_identifier) {
        uuid
    } else {
        storage
            .find_account_by_name(&account_identifier)
            .ok_or(format!("Account '{}' not found", account_identifier))?
            .id
    };

    storage.delete_account(&account_id)?;
    storage.save()?;

    println!("Account deleted successfully");
    Ok(())
}

fn deposit(account_identifier: String, amount: f64) -> Result<(), String> {
    let mut storage = Storage::load()?;

    let account_id = if let Ok(uuid) = Uuid::parse_str(&account_identifier) {
        uuid
    } else {
        storage
            .find_account_by_name(&account_identifier)
            .ok_or(format!("Account '{}' not found", account_identifier))?
            .id
    };

    let account = storage
        .get_account_mut(&account_id)
        .ok_or("Account not found")?;

    account.deposit(amount)?;

    println!(
        "Deposited {:.2} {} to '{}'",
        amount, account.currency, account.name
    );
    println!("New balance: {:.2} {}", account.balance, account.currency);

    storage.save()?;
    Ok(())
}

fn withdraw(account_identifier: String, amount: f64) -> Result<(), String> {
    let mut storage = Storage::load()?;

    let account_id = if let Ok(uuid) = Uuid::parse_str(&account_identifier) {
        uuid
    } else {
        storage
            .find_account_by_name(&account_identifier)
            .ok_or(format!("Account '{}' not found", account_identifier))?
            .id
    };

    let account = storage
        .get_account_mut(&account_id)
        .ok_or("Account not found")?;

    account.withdraw(amount)?;

    println!(
        "Withdrew {:.2} {} from '{}'",
        amount, account.currency, account.name
    );
    println!("New balance: {:.2} {}", account.balance, account.currency);

    storage.save()?;
    Ok(())
}

fn show_account(account_identifier: String) -> Result<(), String> {
    let storage = Storage::load()?;

    let account = if let Ok(uuid) = Uuid::parse_str(&account_identifier) {
        storage.get_account(&uuid).ok_or("Account not found")?
    } else {
        storage
            .find_account_by_name(&account_identifier)
            .ok_or(format!("Account '{}' not found", account_identifier))?
    };

    println!("\nAccount Details:");
    println!("  ID:         {}", account.id);
    println!("  Name:       {}", account.name);
    println!("  Balance:    {:.2} {}", account.balance, account.currency);
    println!(
        "  Created:    {}",
        account.created_at.format("%Y-%m-%d %H:%M:%S")
    );

    Ok(())
}
