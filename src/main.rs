mod utils;
use rust_decimal::prelude::*;
use serde_json::json;
use std::env;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::{error::Error, process};
use utils::ledger_json::Ledger;

static mut USED_ADDRESS: Vec<String> = vec![];

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    let csv_file = args[1].to_string();
    let args_string = args[2..].join(" ").to_string();
    if !args_string.contains('i') {
        println!("Please provide parameter i following with file_name");
        println!("~command~ i invalid_address.csv");
        process::exit(0);
    }
    for (count, args_to_proccess) in args_string.split('w').enumerate() {
        let args_to_proccess_vec: Vec<String> = args_to_proccess
            .split_whitespace() // Use split_whitespace to split at whitespace
            .map(String::from) // Convert each substring to String
            .collect();
        println!("\nProcessing command {}\n", count + 1);
        init(&csv_file, args_to_proccess_vec).await?;
    }
    let used = unsafe { mem::take(&mut USED_ADDRESS) };
    let non_valid_addresses = set_difference(
        get_addresses_from_csv(csv_file.to_string()).expect("Error creating list of addresses"),
        &used,
    );

    let mut file = create_csv_file(
        if &args.last().unwrap()[&args.last().unwrap().trim().len() - 4..] == ".csv" {
            args.last().unwrap()
        } else {
            println!("Please provide output for invalid with .csv file");
            process::exit(0);
        },
    );
    for address in non_valid_addresses.iter() {
        file.write_all(format!("{},\n", address).as_bytes())
            .unwrap();
    }
    Ok(())
}

async fn init(csv_file: &str, args: Vec<String>) -> Result<(), reqwest::Error> {
    let (min, max, command_filter, (is_staking, address)) = parse_command(&args);
    let addresses = valid_addresses(csv_file);
    let body = json!({
        "addresses": addresses,
        "aggregation_level": "Vault",
        "opt_ins": {
            "ancestor_identities": true,
            "component_royalty_vault_balance": true,
            "package_royalty_vault_balance": true,
            "non_fungible_include_nfids": true,
            "explicit_metadata": [
                "name"
            ]
        }
    });
    let response = reqwest::Client::new()
        .post("https://gateway.radix.live/state/entity/details")
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let mut file = create_csv_file(
            if &args.last().unwrap()[&args.last().unwrap().trim().len() - 4..] == ".csv" {
                args.last().unwrap()
            } else {
                println!("Please provide output .csv file");
                process::exit(0);
            },
        );
        let ledger_data: Ledger = serde_json::from_str(&body).expect("failed to load json data");
        if is_staking {
            get_staking_info(ledger_data, command_filter, min, max, &mut file);
        } else {
            get_with_resource_address_info(
                ledger_data,
                address,
                command_filter,
                min,
                max,
                &mut file,
            );
        }
    } else {
        println!("Error: {}", response.status());
        process::exit(0);
    }

    Ok(())
}

fn set_difference(vec1: Vec<String>, vec2: &[String]) -> Vec<String> {
    vec1.into_iter()
        .filter(|item| !vec2.contains(item))
        .collect()
}
fn valid_addresses(csv_file: &str) -> Vec<String> {
    let used = unsafe { mem::take(&mut USED_ADDRESS) };
    let addresses = set_difference(
        get_addresses_from_csv(csv_file.to_string()).expect("Error creating list of addresses"),
        &used,
    );
    unsafe {
        USED_ADDRESS = used.to_owned();
    }
    addresses
}
fn get_with_resource_address_info(
    ledger_data: Ledger,
    address: String,
    command_filter: String,
    min: Decimal,
    max: Decimal,
    file: &mut File,
) {
    for account in ledger_data.get_addresses() {
        let mut token_sumation = Decimal::ZERO;
        println!("\nCurrent address : {:?}.", &account.address);
        for token in account.get_tokens_owned() {
            if token.get_resource_address() == address {
                println!("Total own : {:?}", &token.get_amount());
                token_sumation += Decimal::from_str(token.get_amount()).unwrap();
                break;
            }
        }
        let condition = command_filter == "min" && token_sumation >= min
            || command_filter == "between" && token_sumation >= min && token_sumation <= max;

        if condition {
            println!("{} VALID!", account.address);
            file.write_all(format!("{},\n", account.address).as_bytes())
                .unwrap();
            unsafe {
                USED_ADDRESS.push(account.address.to_string());
            }
        }
    }
}

fn get_staking_info(
    ledger_data: Ledger,
    command_filter: String,
    min: Decimal,
    max: Decimal,
    file: &mut File,
) {
    for account in ledger_data.get_addresses() {
        let mut token_sumation = Decimal::ZERO;
        println!("\nCurrent address : {:?}.", &account.address);
        for token in account.get_tokens_owned() {
            if token.is_lsu() {
                println!("Staking : {:?}", &token.get_amount());
                token_sumation += Decimal::from_str(token.get_amount()).unwrap();
            }
        }
        println!("Total staking : {:?}", token_sumation);
        let condition = command_filter == "min" && token_sumation >= min
            || command_filter == "between" && token_sumation >= min && token_sumation <= max;

        if condition {
            println!("{} VALID!", account.address);
            file.write_all(format!("{},\n", account.address).as_bytes())
                .unwrap();
            unsafe {
                USED_ADDRESS.push(account.address.to_string());
            }
        }
        println!();
    }
}
fn get_addresses_from_csv(csv: String) -> Result<Vec<String>, Box<dyn Error>> {
    let mut list_addresses: Vec<String> = Vec::new();
    let mut rdr = csv::Reader::from_path(csv)?;
    for result in rdr.records() {
        let record = result?;
        list_addresses.push(record.as_slice().to_string());
    }
    Ok(list_addresses)
}

fn create_csv_file(file_name: &str) -> File {
    let mut file = File::create(file_name)
        .map_err(|err| {
            println!("Error creating file {}.", err);
        })
        .unwrap();
    file.write_all("address,\n".as_bytes()).unwrap();
    file
}

fn parse_command(commands: &Vec<String>) -> (Decimal, Decimal, String, (bool, String)) {
    let (mut min, mut max, command, command_filter) = (
        Decimal::ZERO,
        Decimal::ZERO,
        commands[0].trim(),
        commands[1].trim(),
    );
    let (mut is_staking, mut address) = (false, String::new());
    if commands.len() > 3 {
        if command == "staking" {
            is_staking = true;
        } else if &command[0..13] == "resource_rdx1" && command.len() == 67 {
            address = command.to_string();
        } else {
            println!(
                "please specify valid command, it's either 'staking' or valid resource address"
            );
            process::exit(0);
        }

        if command_filter != "min" && command_filter != "between" {
            println!("Unknown command: {}.", command_filter);
            println!("please choose only 'min' or 'between'.");
            process::exit(0)
        }
        if command_filter == "min" {
            min = Decimal::from_str(commands[2].trim()).expect("Provide valid number minimum");
        } else if command_filter == "between" {
            min = Decimal::from_str(commands[2].trim()).expect("Provide valid number minimum");
            max = Decimal::from_str(commands[3].trim()).expect("Provide valid number maximum");
            if max <= min {
                println!("max must be greater than min");
                process::exit(0);
            }
        }
    } else {
        println!("please specify valid command");
        process::exit(0);
    };
    (min, max, command_filter.to_string(), (is_staking, address))
}
