use cosmwasm_std::{
    entry_point, DepsMut, Env, MessageInfo, Response, StdResult, ContractResult, Deps, to_vec, StdError, SystemResult, to_binary, Binary, WasmQuery, QueryRequest, WasmMsg, CosmosMsg, Uint128
};

use injective_cosmwasm::{InjectiveQuerier, InjectiveQueryWrapper};
use crate::state::{ShouterMessage, Config, CONFIG, ShoutBackMessage};
use xca::wormhole::{WormholeExecuteMsg, WormholeQueryMsg, ParsedVAA};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {

    CONFIG.save(deps.storage, &Config{
        wormhole_contract: deps.api.addr_validate(&msg.wormhole_contract)?,
    })?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Execute{msg} => {
            Ok(Response::new().add_message(msg))
        },
        ExecuteMsg::SubmitVaa{vaa} => {
            let config: Config = CONFIG.load(deps.storage)?;

            let vaa = parse_vaa(deps.as_ref(), env.block.time.seconds(), &vaa)?;
            let message_in: ShouterMessage = ShouterMessage::deserialize(&vaa.payload)?;

            let message: ShoutBackMessage = ShoutBackMessage{
                best_bid: Uint128::from(7u128<<93),
                best_ask: Uint128::from(9u128<<93),
            };

            let msg_out: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute{
                contract_addr: config.wormhole_contract.into(),
                funds: vec![],
                msg: to_binary(&WormholeExecuteMsg::PostMessage{
                    message: message.serialize().into(),
                    nonce: 420u32,
                })?,
            });

            Ok(Response::new().add_attributes(vec![("shout_in", message_in.payload[0].to_string()), ("best_bid", message.best_bid.to_string()), ("best_ask", message.best_ask.to_string())]).add_message(msg_out))
        },
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<InjectiveQueryWrapper>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let querier = InjectiveQuerier::new(&deps.querier);

    match msg {
        QueryMsg::Query {msg} => {
            let raw = to_vec(&msg).map_err(|serialize_err| {
                StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
            })?;

            match deps.querier.raw_query(&raw) {
                SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                    "Querier system error: {}",
                    system_err
                ))),
                SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
                    "Querier contract error: {}",
                    contract_err
                ))),
                SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
            }
        },

        QueryMsg::SubaccountDeposit { subaccount_id, denom} => Ok(to_binary(&querier.query_subaccount_deposit(subaccount_id, denom)?)?),
        QueryMsg::SpotMarket {market_id} => Ok(to_binary(&querier.query_spot_market(market_id)?)?),
        QueryMsg::TraderSpotOrders { market_id, subaccount_id} => Ok(to_binary(&querier.query_trader_spot_orders(market_id, subaccount_id)?)?),
        QueryMsg::TraderSpotOrdersToCancelUpToAmount {market_id,subaccount_id,base_amount,quote_amount,strategy,reference_price,} => Ok(to_binary(&querier.query_spot_orders_to_cancel_up_to_amount(market_id, subaccount_id, base_amount, quote_amount, strategy, reference_price)?)?),
        QueryMsg::TraderDerivativeOrdersToCancelUpToAmount {market_id,subaccount_id,quote_amount,strategy,reference_price} => Ok(to_binary(&querier.query_derivative_orders_to_cancel_up_to_amount(market_id, subaccount_id, quote_amount, strategy, reference_price)?)?),
        QueryMsg::DerivativeMarket {market_id} => Ok(to_binary(&querier.query_derivative_market(market_id)?)?),
        //QueryMsg::SubaccountPositions {subaccount_id,} => Ok(to_binary(&querier.query_subaccount_positions(subaccount_id)?)?),
        QueryMsg::SubaccountPositionInMarket {market_id,subaccount_id,} => Ok(to_binary(&querier.query_vanilla_subaccount_position(market_id, subaccount_id)?)?),
        QueryMsg::SubaccountEffectivePositionInMarket {market_id,subaccount_id,} => Ok(to_binary(&querier.query_effective_subaccount_position(market_id, subaccount_id)?)?),
        QueryMsg::TraderDerivativeOrders {market_id,subaccount_id,} => Ok(to_binary(&querier.query_trader_derivative_orders(market_id, subaccount_id)?)?),
        QueryMsg::TraderTransientSpotOrders {market_id,subaccount_id,} => Ok(to_binary(&querier.query_trader_transient_spot_orders(market_id, subaccount_id)?)?),
        QueryMsg::TraderTransientDerivativeOrders {market_id,subaccount_id,} => Ok(to_binary(&querier.query_trader_transient_derivative_orders(market_id, subaccount_id)?)?),
        QueryMsg::PerpetualMarketInfo {market_id,} => Ok(to_binary(&querier.query_perpetual_market_info(market_id)?)?),
        QueryMsg::PerpetualMarketFunding {market_id,} => Ok(to_binary(&querier.query_perpetual_market_funding(market_id)?)?),
        //QueryMsg::MarketVolatility {market_id,trade_history_options,} => Ok(to_binary(&querier.query_market_volatility(market_id, trade_history_options)?)?),
        QueryMsg::SpotMarketMidPriceAndTob {market_id,} => Ok(to_binary(&querier.query_spot_market_mid_price_and_tob(market_id)?)?),
        QueryMsg::DerivativeMarketMidPriceAndTob {market_id,} => Ok(to_binary(&querier.query_derivative_market_mid_price_and_tob(market_id)?)?),
        //QueryMsg::OracleVolatility {base_info,quote_info,oracle_history_options} => Ok(to_binary(&querier.query_oracle_volatility(base_info, quote_info, oracle_history_options)?)?),

    }
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

