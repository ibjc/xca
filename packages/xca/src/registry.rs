pub struct Config{
  pub wormhole_core_contract: String,
  pub x_account_factory: String,
  pub wormhole_chain_ids: Vec<Chain>,
  pub x_account_code_id: u64,
}

pub struct Chain{
  pub name: String,
  pub wormhole_id: u8,
}