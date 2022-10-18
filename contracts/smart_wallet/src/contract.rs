#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, QueryRequest, Response, StdError, StdResult, Uint128,
    WasmMsg, WasmQuery, BankQuery, AllBalanceResponse, Coin, Decimal,
};
use xca::wormhole::{ConfigResponse, ExecuteMsg, QueryMsg, InstantiateMsg, WormholeExecuteMsg, WormholeQueryMsg, ParsedVAA, AccountInfo};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse as Cw20BalanceResponse};
use crate::state::{Config, CONFIG, VAA_ARCHIVE, TokenBridgeMessage, State, STATE};
use std::str::FromStr;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        wormhole_contract: deps.api.addr_validate(&msg.owner)?,
        accounts: msg.accounts,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            wormhole_contract,
        } => update_config(
            deps,
            info,
            owner,
            wormhole_contract,
        ),

        ExecuteMsg::UpsertAccount{
            account_info,
        } => Ok(Response::new()),

        ExecuteMsg::DeleteAccount{
            chain_id,
            address,
        } => Ok(Response::new()),

        //dispatch arbitrary data to wormhole; recipient is itself
        ExecuteMsg::WormholeDispatch {
            payload,
            nonce,
            receiver,
        } => wormhole_dispatch(deps, info, payload, nonce, receiver),

        ExecuteMsg::WormholeReceive {vaa} => wormhole_receive(deps, env, info, vaa),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn wormhole_dispatch(
    deps: DepsMut,
    info: MessageInfo,
    payload: Binary,
    nonce: Option<u32>,
    receiver: Option<AccountInfo>,
) -> StdResult<Response> {

    let config: Config = CONFIG.load(deps.storage)?;

    let message_payload = TokenBridgeMessage{
        payload: payload.into()
    };

    let message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: config.wormhole_contract.into(),
        funds: vec![],
        msg: to_binary(&WormholeExecuteMsg::PostMessage{
            message: to_binary(&message_payload.serialize())?,
            nonce: nonce.unwrap_or(69u32),
        })?
    });

    Ok(Response::new().add_message(message))
}

#[allow(clippy::too_many_arguments)]
pub fn wormhole_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vaa: Binary
) -> StdResult<Response> {
    let (vaa, message) = parse_and_archive_vaa(deps, env, &vaa)?;

    //
    

    Ok(Response::new())
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    wormhole_contract: Option<String>,
) -> StdResult<Response> {


    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        wormhole_contract: config.wormhole_contract.to_string(),
        accounts: config.accounts,
    })
}



fn parse_and_archive_vaa(
    deps: DepsMut,
    env: Env,
    data: &Binary,
) -> StdResult<(ParsedVAA, TokenBridgeMessage)> {
    let vaa = parse_vaa(deps.as_ref(), env.block.time.seconds(), data)?;

    if !VAA_ARCHIVE.may_load(deps.storage, vaa.hash.as_slice())?.unwrap_or(false) {
        return Err(StdError::generic_err("VAA already executed"));
    }
    VAA_ARCHIVE.save(deps.storage, vaa.hash.as_slice(), &true)?;

    let message = TokenBridgeMessage::deserialize(&vaa.payload)?;
    Ok((vaa, message))
}

fn parse_vaa(deps: Deps, block_time: u64, data: &Binary) -> StdResult<ParsedVAA> {
     let config: Config = CONFIG.load(deps.storage)?;

    let vaa: ParsedVAA = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.wormhole_contract.into(),
        msg: to_binary(&WormholeQueryMsg::VerifyVAA {
            vaa: data.clone(),
            block_time,
        })?,
    }))?;
    Ok(vaa)
}

