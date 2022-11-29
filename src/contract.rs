#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Addr;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_std::{BankMsg, SubMsg};
use cw2::set_contract_version;
//use cw_multi_test::Contract;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:simple-option";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.expiry <= _env.block.height {
        return Err(ContractError::Std(cosmwasm_std::StdError::GenericErr {
            msg: "Wrong expiry".to_string(),
        }));
    }
    let state = State {
        collateral: info.funds,
        expires: msg.expiry,
        counter_offer: msg.counter_offer,
        creator: info.sender.clone(),
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.expiry.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient } => {
            execute::execute_transfer(deps, _env, info, recipient)
        }
        ExecuteMsg::Execute {} => execute::execute_option(deps, _env, info),
        ExecuteMsg::Burn {} => execute::execute_burn(deps, _env, info),
    }
}

pub mod execute {
    use super::*;

    pub fn execute_transfer(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        recipient: Addr,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.owner = recipient;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    pub fn execute_option(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        //Ensure message sender is the owner
        let state = STATE.load(deps.storage)?;
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }

        //ensure not expired
        if state.expires <= _env.block.height {
            return Err(ContractError::Std(cosmwasm_std::StdError::GenericErr {
                msg: "Message Expired".to_string(),
            }));
        }

        //ensure sending proper counter offer
        if info.funds != state.counter_offer {
            return Err(ContractError::Std(cosmwasm_std::StdError::GenericErr {
                msg: format!("Wrong Counter offer ").to_string(),
            }));
        }
        let mut msg = SubMsg::new(BankMsg::Send {
            to_address: (state.creator.to_string()),
            amount: (state.counter_offer),
        });

        //let mut res = Context::new();

        //release counter offer to creator

        //release collateral to sender

        //delete the option

        // STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        //     if info.sender != state.owner {
        //         return Err(ContractError::Unauthorized {});
        //     }
        //     state.count = count;
        //     Ok(state)
        // })?;
        // let rsp = Response::new();
        // rsp.add_attribute("action", "reset");
        // rsp.add_submessage(msg);

        Ok(Response::new()
            .add_attribute("action", "reset")
            .add_submessage(msg))
    }

    pub fn execute_burn(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        if state.expires > _env.block.height {
            return Err(ContractError::Std(cosmwasm_std::StdError::GenericErr {
                msg: "Message Expired".to_string(),
            }));
        }
        if !info.funds.is_empty() {
            return Err(ContractError::Std(cosmwasm_std::StdError::GenericErr {
                msg: "Not empty funds".to_string(),
            }));
        }
        let msg = SubMsg::new(BankMsg::Send {
            to_address: (_env.contract.address.to_string()),
            amount: (state.counter_offer),
        });

        // let rsp = Response::new();
        Ok(Response::new()
            .add_attribute("action", "reset")
            .add_submessage(msg))
        //rsp.add_submessage(msg);
        //Ok(rsp)
        //todo!()
        //todo!()
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query::contract_data(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn contract_data(deps: Deps) -> StdResult<ConfigResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(state)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies();

//         let msg = InstantiateMsg { counter_offer: 17 , expiry:123 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: GetCountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies();

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: GetCountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies();

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

//         // should now be 5
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: GetCountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
