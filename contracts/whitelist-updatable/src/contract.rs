use crate::state::{Config, CONFIG, TOTAL_ADDRESS_COUNT, WHITELIST};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:whitelist-updatable";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        admin: info.sender,
        per_address_limit: msg.per_address_limit,
    };

    // remove duplicate addresses
    msg.addresses.sort_unstable();
    msg.addresses.dedup();

    let mut count = 0u64;
    for address in msg.addresses.into_iter() {
        let addr = deps.api.addr_validate(&address.clone())?;
        WHITELIST.save(deps.storage, addr, &0u64)?;
        count += 1;
    }

    TOTAL_ADDRESS_COUNT.save(deps.storage, &count)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, info, new_admin),
        ExecuteMsg::AddAddresses { addresses } => execute_add_addresses(deps, info, addresses),
        ExecuteMsg::RemoveAddresses { addresses } => {
            execute_remove_addresses(deps, info, addresses)
        }
        ExecuteMsg::ProcessAddress { address } => execute_process_address(deps, info, address),
        ExecuteMsg::UpdatePerAddressLimit { limit } => {
            execute_update_per_address_limit(deps, info, limit)
        }
        ExecuteMsg::Purge {} => execute_purge(deps, info),
    }
}

pub fn execute_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    config.admin = deps.api.addr_validate(&new_admin)?;
    CONFIG.save(deps.storage, &config)?;
    let event = Event::new("update_admin")
        .add_attribute("new_admin", config.admin)
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_add_addresses(
    deps: DepsMut,
    info: MessageInfo,
    mut addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut count = TOTAL_ADDRESS_COUNT.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // dedupe
    addresses.sort_unstable();
    addresses.dedup();

    for address in addresses.into_iter() {
        let addr = deps.api.addr_validate(&address.clone())?;
        WHITELIST.save(deps.storage, addr, &0u64)?;
        count += 1;
    }

    TOTAL_ADDRESS_COUNT.save(deps.storage, &count)?;

    let event = Event::new("add_addresses")
        .add_attribute("new-count", count.to_string())
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_remove_addresses(
    deps: DepsMut,
    info: MessageInfo,
    mut addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut count = TOTAL_ADDRESS_COUNT.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // dedupe
    addresses.sort_unstable();
    addresses.dedup();

    for address in addresses.into_iter() {
        let addr = deps.api.addr_validate(&address.clone())?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            WHITELIST.remove(deps.storage, addr);
            count -= 1;
        }
    }

    let event = Event::new("remove_addresses")
        .add_attribute("new-count", count.to_string())
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_process_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let addr = deps.api.addr_validate(&address.clone())?;
    if !WHITELIST.has(deps.storage, addr.clone()) {
        return Err(ContractError::AddressNotFound {});
    }

    let count = WHITELIST.load(deps.storage, addr.clone())?;
    WHITELIST.save(deps.storage, addr.clone(), &(count + 1))?;

    let event = Event::new("process_address")
        .add_attribute("address", address)
        .add_attribute("mint-count", (count + 1).to_string())
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    info: MessageInfo,
    limit: u32,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    config.per_address_limit = limit;
    CONFIG.save(deps.storage, &config)?;

    let event = Event::new("update_per_address_limit")
        .add_attribute("new-limit", limit.to_string())
        .add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

pub fn execute_purge(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let keys: Vec<Addr> = WHITELIST
        .keys(deps.as_ref().storage, None, None, Order::Ascending)
        .map(|x| x.unwrap())
        .collect::<Vec<Addr>>();
    keys.into_iter().for_each(|key| {
        WHITELIST.remove(deps.storage, key);
    });

    let event = Event::new("purge").add_attribute("sender", info.sender);
    Ok(Response::new().add_event(event))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
