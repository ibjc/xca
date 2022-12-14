#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, StdError, Coin, BankMsg, CosmosMsg, to_binary};
// use cw2::set_contract_version;

use xca::staking::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, StateResponse};
use crate::state::{Config, CONFIG, State, STATE, STAKERS};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    CONFIG.save(deps.storage, &Config{
        denom_name: msg.denom_name,
    })?;

    STATE.save(deps.storage, &State{
        total_staked: Uint128::zero(),
    })?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Stake{} => {

            let config: Config = CONFIG.load(deps.storage)?;

            //confirm sent funds denom
            let amount = info 
            .funds
            .iter()
            .find(|x| x.denom == config.denom_name && x.amount > Uint128::zero())
            .ok_or_else(|| {
                StdError::generic_err(format!("Expected stakable denom not sent"))
            })?.amount;

            //try fetch sender's staker entry
            let staker_amount: Uint128 = STAKERS
                .may_load(deps.storage, &info.sender.clone())?
                .unwrap_or(Uint128::zero());

            //save stakers and state (funds already in contract)
            STAKERS.save(deps.storage, &info.sender.clone(), &(staker_amount + amount))?;

            let mut state: State = STATE.load(deps.storage)?;
            state.total_staked += amount;
            STATE.save(deps.storage, &state)?;

            //exit
            Ok(Response::new())
        },
        ExecuteMsg::Unstake{} => {

            let config: Config = CONFIG.load(deps.storage)?;

            //fetch sender amount
            let staker_amount: Uint128 = STAKERS
                .may_load(deps.storage, &info.sender.clone())?
                .unwrap_or(Uint128::zero());

            //fabricate bank send
            let send_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send{
                to_address: info.sender.clone().into(),
                amount: vec![
                    Coin{
                        denom: config.denom_name,
                        amount: staker_amount,
                    }
                ],
            });
            
            //exit and dispatch
            
            Ok(Response::new().add_message(send_msg))
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg{
        QueryMsg::Config {} => {
            let config: Config = CONFIG.load(deps.storage)?;
    
            Ok(to_binary(&ConfigResponse{
                denom_name: config.denom_name,
            })?)
        },

        QueryMsg::State{} => {
            let state: State = STATE.load(deps.storage)?;
    
            Ok(to_binary(&StateResponse{
                total_staked: state.total_staked,
            })?)
        }
    }
}

#[cfg(test)]
mod tests {}
