#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
// use cw2::set_contract_version;

use crate::state::{Config, CONFIG};
use xca::registry::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

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
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            chain_id_here: msg.chain_id_here,
            x_account_code_id: msg.x_account_code_id,
            chain_info: vec![],
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            x_account_code_id,
        } => {
            let mut config: Config = CONFIG.load(deps.storage)?;

            // TODO: access check - if config.wormhole_core_contract != owner

            if let Some(x_account_code_id) = x_account_code_id {
                config.x_account_code_id = x_account_code_id;
            }

            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new())
        }
        ExecuteMsg::UpsertChainInfo { chain_info } => {
            let mut config: Config = CONFIG.load(deps.storage)?;

            // TODO: access check - if config.wormhole_core_contract != owner

            let position = config
                .chain_info
                .iter()
                .position(|x| x.wormhole_id == chain_info.wormhole_id);

            if let Some(position) = position {
                config.chain_info.remove(position);
            }

            config.chain_info.push(chain_info);

            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config: Config = CONFIG.load(deps.storage)?;

            Ok(to_binary(&ConfigResponse {
                chain_id_here: config.chain_id_here,
                x_account_code_id: config.x_account_code_id,
                chain_info: config.chain_info,
            })?)
        }
    }
}

#[cfg(test)]
mod tests {}
