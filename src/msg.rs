use crate::state::State;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cosmwasm_std::Coin;

#[cw_serde]
pub struct InstantiateMsg {
    //owner and creator
    //collateralcomes from env
    pub counter_offer: Vec<Coin>,
    pub expiry: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    ///Owner can transfer to a new owner
    Transfer { recipient: Addr },
    ///Owner can post counter offer on unexpired option to execute and get the collateral
    Execute {},
    /// Burn will release collateral if expired
    Burn {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(ConfigResponse)]
    GetState {},
}

// We define a custom struct for each query response
//#[cw_serde]
pub type ConfigResponse = State;
