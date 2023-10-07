use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Values {
    pub raw_hex: String,
}
#[derive(Debug, Deserialize)]
pub struct Data {
    pub value: Values,
}
#[derive(Debug, Deserialize)]
pub struct ResourceMetadata {
    pub items: Vec<Data>,
}
#[derive(Debug, Deserialize)]
pub struct Vault {
    pub amount: String,
}
#[derive(Debug, Deserialize)]
pub struct Vaults {
    pub items: Vec<Vault>,
}
#[derive(Debug, Deserialize)]
pub struct ResourceData {
    pub vaults: Vaults,
    pub explicit_metadata: ResourceMetadata,
}
impl ResourceData {
    pub fn get_token_metadata(&self) -> &Values {
        &self
            .explicit_metadata
            .items
            .first()
            .expect("no metadata")
            .value
    }
    pub fn is_lsu(&self) -> bool {
        self.get_token_metadata().raw_hex == "5c2200010c124c6971756964205374616b6520556e697473"
    }
    pub fn get_staked_amount(&self) -> &str {
        self.vaults
            .items
            .first()
            .expect("failed to parse amount")
            .amount
            .trim()
    }
}
#[derive(Debug, Deserialize)]
pub struct FungibleResources {
    pub items: Vec<ResourceData>,
}
#[derive(Debug, Deserialize)]
pub struct Account {
    pub address: String,
    pub fungible_resources: FungibleResources,
}
impl Account {
    pub fn get_tokens_owned(&self) -> &Vec<ResourceData> {
        &self.fungible_resources.items
    }
}
#[derive(Debug, Deserialize)]
pub struct State {
    pub proposer_round_timestamp: String,
}
#[derive(Debug, Deserialize)]
pub struct Ledger {
    pub ledger_state: State,
    pub items: Vec<Account>,
}
impl Ledger {
    pub fn get_addresses(&self) -> &Vec<Account> {
        &self.items
    }
}
