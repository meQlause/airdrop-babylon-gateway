mod utils;
use rust_decimal::prelude::*;
use serde_json::json;
use std::env;
use std::fs::File;
use std::io::Write;
use std::{error::Error, process};
use utils::ledger_json::Ledger;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    let (mut min, mut max) = (Decimal::ZERO, Decimal::ZERO);
    if args.len() > 3 {
        if args[2].trim() != "min" && args[2].trim() != "between" {
            println!("Unknown command: {}.", args[2]);
            println!("please choose only 'min' or 'between'.");
            process::exit(0)
        }
        if args[2].trim() == "min" {
            min = Decimal::from_str(args[3].trim()).expect("Provide valid number minimum");
        } else if args[2].trim() == "between" {
            min = Decimal::from_str(args[3].trim()).expect("Provide valid number minimum");
            max = Decimal::from_str(args[4].trim()).expect("Provide valid number maximum");
        }
    } else {
        println!("please specify valid command");
        process::exit(0);
    };
    let addresses = get_addresses_from_csv(args[1].to_string())
        .map_err(|err| {
            println!("Can not add address {}.", err);
            println!("Ensure that your csv file has the same format as ours.");
            process::exit(0);
        })
        .unwrap();
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
        let mut file = create_csv_file_for_eligible_address(
            if &args.last().unwrap()[&args.last().unwrap().trim().len() - 4..] == ".csv" {
                args.last().unwrap()
            } else {
                println!("Please provide output .csv file");
                process::exit(0);
            },
        );
        let ledger_data: Ledger = serde_json::from_str(&body).expect("failed to load json data");

        for account in ledger_data.get_addresses() {
            let mut lsu_sumation = Decimal::ZERO;
            println!("\nCurrent address : {:?}.", &account.address);
            for token in account.get_tokens_owned() {
                if token.is_lsu() {
                    println!("Staking : {:?}", &token.get_staked_amount());
                    lsu_sumation += Decimal::from_str(token.get_staked_amount()).unwrap();
                }
            }
            println!("Total staking : {:?}", lsu_sumation);
            let condition = args[2].trim() == "min" && lsu_sumation >= min
                || args[2].trim() == "between" && lsu_sumation >= min && lsu_sumation <= max;

            if condition {
                println!("{} VALID!", account.address);
                file.write_all(format!("{},\n", account.address).as_bytes())
                    .unwrap();
            }
            println!();
        }
    } else {
        println!("Error: {}", response.status());
        process::exit(0);
    }

    Ok(())
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

fn create_csv_file_for_eligible_address(file_name: &str) -> File {
    let mut file = File::create(file_name)
        .map_err(|err| {
            println!("Error creating file {}.", err);
        })
        .unwrap();
    file.write_all("address,\n".as_bytes()).unwrap();
    file
}
