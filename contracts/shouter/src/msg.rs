use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CosmosMsg, QueryRequest, Empty, Binary};
use injective_math::FPDecimal;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub wormhole_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Execute{msg: CosmosMsg},
    SubmitVaa{vaa: Binary},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg{
    Query{msg: QueryRequest<Empty>},
    SubaccountDeposit {
        subaccount_id: String,
        denom: String,
    },
    SpotMarket {
        market_id: String,
    },
    TraderSpotOrders {
        market_id: String,
        subaccount_id: String,
    },
    TraderSpotOrdersToCancelUpToAmount {
        market_id: String,
        subaccount_id: String,
        base_amount: FPDecimal,
        quote_amount: FPDecimal,
        strategy: i32,
        reference_price: Option<FPDecimal>,
    },
    TraderDerivativeOrdersToCancelUpToAmount {
        market_id: String,
        subaccount_id: String,
        quote_amount: FPDecimal,
        strategy: i32,
        reference_price: Option<FPDecimal>,
    },
    DerivativeMarket {
        market_id: String,
    },
    SubaccountPositionInMarket {
        market_id: String,
        subaccount_id: String,
    },
    SubaccountEffectivePositionInMarket {
        market_id: String,
        subaccount_id: String,
    },
    TraderDerivativeOrders {
        market_id: String,
        subaccount_id: String,
    },
    TraderTransientSpotOrders {
        market_id: String,
        subaccount_id: String,
    },
    TraderTransientDerivativeOrders {
        market_id: String,
        subaccount_id: String,
    },
    PerpetualMarketInfo {
        market_id: String,
    },
    PerpetualMarketFunding {
        market_id: String,
    },
    SpotMarketMidPriceAndTob {
        market_id: String,
    },
    DerivativeMarketMidPriceAndTob {
        market_id: String,
    },
}