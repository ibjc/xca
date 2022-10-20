#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
    SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use xca::account::Config;
use xca::byte_utils::ByteUtils;
use xca::error::ContractError as XcaContractError;
use xca::messages::{AccountInfo, ParsedVAA, WormholeMessage};
use xca::registry::{ConfigResponse as RegistryConfigResponse, QueryMsg as RegistryQueryMsg};
use xca::request::Request;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::CONFIG;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:account";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const POST_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // TODO gate access for who can instantiate
    // TODO validate addrs
    deps.api.addr_validate(&msg.x_chain_registry_address)?;
    CONFIG.save(
        deps.storage,
        &Config {
            x_chain_registry: msg.x_chain_registry_address,
            admin: msg.admin,
            master: msg.master,
            slave: msg.slave,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract-name", CONTRACT_NAME)
        .add_attribute("contract-version", CONTRACT_VERSION))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Call {
            msg_type,       // e.g. ExecuteMsg, QueryMsg, InstatiateMsg, MigrateMsg. null => ExecuteMsg
            msg,            // base64-encoded stringified JSON
            destination,    // address registry required
            receive_caller, // optional general relayer usage
            is_response_expected, // give back Ok(x => VAA)
            execution_dependency, // wormhole_message.sequence here
        } => execute_call(
            deps,
            info,
            msg_type,
            msg,
            destination,
            receive_caller,
            is_response_expected,
            execution_dependency,
        ),
        ExecuteMsg::BroadcastCall { request } => execute_broadcast_call(deps, info, request),
        ExecuteMsg::FinishCall { vaas } => execute_finish_call(deps, info, vaas),
        ExecuteMsg::UpdateConfig {
            x_chain_registry,
            admin,
            master,
            slave,
        } => execute_update_config(deps, info, x_chain_registry, admin, master, slave),
    }
}

pub fn execute_call(
    deps: DepsMut,
    info: MessageInfo,
    msg_type: Option<String>,
    msg: Binary,
    destination: AccountInfo,
    receive_caller: Option<String>,
    is_response_expected: Option<bool>,
    execution_dependency: Option<WormholeMessage>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // query registry
    let registry_addr = deps.api.addr_validate(&config.x_chain_registry)?;
    let res: RegistryConfigResponse = deps
        .querier
        .query_wasm_smart(registry_addr, &RegistryQueryMsg::Config {})?;

    // registry config has wormhole address

    // send call to other chain's xaccount
    // pub x_chain_registry: String,   // Updatable by admins
    // pub admin: AccountInfo,         // Can update Config. (chain, addr)
    // pub master: AccountInfo,        // Can accept VAA executions from these. (chain, addr)
    // pub slave: Option<AccountInfo>, //

    let mut submessages = Vec::new();
    submessages.push(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: res.wormhole_core_contract,
            msg,
            funds: vec![],
        }),
        POST_REPLY_ID,
    ));

    Ok(Response::new().add_submessages(submessages))
}

pub fn execute_broadcast_call(
    deps: DepsMut,
    info: MessageInfo,
    request: Request,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn execute_finish_call(
    deps: DepsMut,
    info: MessageInfo,
    vaas: Vec<Binary>,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    x_chain_registry: String,
    admin: AccountInfo,
    master: AccountInfo,
    slave: Option<AccountInfo>,
) -> Result<Response, ContractError> {
    // TODO gate access
    // if config.admin.address != info.sender {
    //     return Err(ContractError::Unauthorized {});
    // }

    // TODO sanitize and validate user input values
    CONFIG.save(
        deps.storage,
        &Config {
            x_chain_registry,
            admin,
            master,
            slave,
        },
    )?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyVAA { vaa, block_time } => to_binary(&query_parse_and_verify_vaa(
            deps,
            vaa.as_slice(),
            block_time,
        )?),
    }
}

pub fn query_parse_and_verify_vaa(
    deps: Deps,
    data: &[u8],
    block_time: u64,
) -> StdResult<ParsedVAA> {
    Ok(parse_and_verify_vaa(deps.storage, data, block_time)?)
}

/// taken from https://github.com/wormhole-foundation/wormhole/blob/dev.v2/cosmwasm/contracts/wormhole/src/contract.rs
/// Parses raw VAA data into a struct and verifies whether it contains sufficient signatures of an
/// active guardian set i.e. is valid according to Wormhole consensus rules
fn parse_and_verify_vaa(
    storage: &dyn Storage,
    data: &[u8],
    block_time: u64,
) -> StdResult<ParsedVAA> {
    let vaa = ParsedVAA::deserialize(data)?;

    if vaa.version != 1 {
        return XcaContractError::InvalidVersion.std_err();
    }

    // Check if VAA with this hash was already accepted
    if vaa_archive_check(storage, vaa.hash.as_slice()) {
        return XcaContractError::VaaAlreadyExecuted.std_err();
    }

    // Load and check guardian set
    let guardian_set = guardian_set_get(storage, vaa.guardian_set_index);
    let guardian_set: xca::wormhole::WormholeQueryMsg =
        guardian_set.or_else(|_| XcaContractError::InvalidGuardianSetIndex.std_err())?;

    if guardian_set.expiration_time != 0 && guardian_set.expiration_time < block_time {
        return XcaContractError::GuardianSetExpired.std_err();
    }
    if (vaa.len_signers as usize) < guardian_set.quorum() {
        return XcaContractError::NoQuorum.std_err();
    }

    // Verify guardian signatures
    let mut last_index: i32 = -1;
    let mut pos = ParsedVAA::HEADER_LEN;

    for _ in 0..vaa.len_signers {
        if pos + ParsedVAA::SIGNATURE_LEN > data.len() {
            return XcaContractError::InvalidVAA.std_err();
        }
        let index = data.get_u8(pos) as i32;
        if index <= last_index {
            return XcaContractError::WrongGuardianIndexOrder.std_err();
        }
        last_index = index;

        let signature = Signature::try_from(
            &data[pos + ParsedVAA::SIG_DATA_POS
                ..pos + ParsedVAA::SIG_DATA_POS + ParsedVAA::SIG_DATA_LEN],
        )
        .or_else(|_| ContractError::CannotDecodeSignature.std_err())?;
        let id = RecoverableId::new(data.get_u8(pos + ParsedVAA::SIG_RECOVERY_POS))
            .or_else(|_| XcaContractError::CannotDecodeSignature.std_err())?;
        let recoverable_signature = RecoverableSignature::new(&signature, id)
            .or_else(|_| XcaContractError::CannotDecodeSignature.std_err())?;

        let verify_key = recoverable_signature
            .recover_verify_key_from_digest_bytes(GenericArray::from_slice(vaa.hash.as_slice()))
            .or_else(|_| XcaContractError::CannotRecoverKey.std_err())?;

        let index = index as usize;
        if index >= guardian_set.addresses.len() {
            return XcaContractError::TooManySignatures.std_err();
        }
        if !keys_equal(&verify_key, &guardian_set.addresses[index]) {
            return XcaContractError::GuardianSignatureError.std_err();
        }
        pos += ParsedVAA::SIGNATURE_LEN;
    }

    Ok(vaa)
}

#[cfg(test)]
mod tests {}
