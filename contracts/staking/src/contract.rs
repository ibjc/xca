#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, StdError};
// use cw2::set_contract_version;

use xca::staking::{ExecuteMsg, InstantiateMsg, QueryMsg};
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

            let mut config: Config = CONFIG.load(deps.storage)?;

            
            Ok(Response::new())
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
