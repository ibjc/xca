use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct ConfigResponse {
  pub denom_name: String,
}

#[cw_serde]
pub struct StateResponse {
  pub total_staked: Uint128,
}

#[cw_serde]
pub struct InstantiateMsg {
  pub denom_name: String,
}

#[cw_serde]
pub enum ExecuteMsg{
  Stake {},
  Unstake {},
}


#[cw_serde]
pub enum QueryMsg{
  Config {},
  State{},
}