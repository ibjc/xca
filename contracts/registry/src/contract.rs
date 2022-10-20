#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
// use cw2::set_contract_version;

use crate::state::{Config, CONFIG};
use xca::registry::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response>  {

    CONFIG.save(deps.storage, &Config{
        wormhole_core_contract: deps.api.addr_validate(&msg.wormhole_core_contract)?,
        x_account_factory: deps.api.addr_validate(&msg.x_account_factory)?,
        wormhole_chain_ids: msg.wormhole_chain_ids,
        x_account_code_id: msg.x_account_code_id,
    })?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response>  {
    match msg {
        ExecuteMsg::UpdateConfig{wormhole_core_contract, x_account_factory, x_account_code_id} => {
            Ok(Response::new())
        },
        ExecuteMsg::UpsertWormholeChainId{chain} => {

            Ok(Response::new())
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg{
        QueryMsg::Config {} => {
            let config: Config = CONFIG.load(deps.storage)?;
    
            Ok(to_binary(&ConfigResponse{
                wormhole_core_contract: config.wormhole_core_contract.to_string(),
                x_account_factory: config.x_account_factory.to_string(),
                wormhole_chain_ids: config.wormhole_chain_ids,
                x_account_code_id: config.x_account_code_id,
            })?)
        }
    }

}

#[cfg(test)]
mod tests {}
