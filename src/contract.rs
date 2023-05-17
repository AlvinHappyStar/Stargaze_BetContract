#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, CosmosMsg
};

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use cw2::{get_contract_version, set_contract_version};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, HashObj, RHistory, RHistoryResponse, FHistory, FHistoryResponse 
};
use cw20::{Balance};
use crate::state::{
    Config, CONFIG, RHISTORY, FHISTORY
};

use crate::util;
use crate::constants;
// Version info, for migration info
const CONTRACT_NAME: &str = "bet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        denom: cw20::Denom::Native(info.funds[0].denom.clone()),
        enabled: true,
        flip_count: 0u64,
        rps_count: 0u64
    };
    
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { owner } => util::execute_update_owner(deps.storage, deps.api, info.sender.clone(), owner),
        ExecuteMsg::UpdateEnabled { enabled } => util::execute_update_enabled(deps.storage, deps.api, info.sender.clone(), enabled),
        ExecuteMsg::Flip { level } => execute_flip(deps, env, info, level),
        ExecuteMsg::Rps { level } => execute_rps(deps, env, info, level),
        ExecuteMsg::Withdraw { amount } => execute_withdraw(deps, env, info, amount)     
    }
}

fn get_winner(p1: u64, p2: u64) -> Option<u8> {
    match (p1, p2) {
        (0, 1) => Some(0),
        (2, 0) => Some(0),
        (1, 2) => Some(0),
        (0, 2) => Some(2),
        (2, 1) => Some(2),
        (1, 0) => Some(2),
        _ => Some(1),
    }
}

pub fn execute_flip(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    level: u64
) -> Result<Response, ContractError> {

    util::check_enabled(deps.storage)?;

    let mut cfg = CONFIG.load(deps.storage)?;

    let balance = Balance::from(info.funds);

    let amount = util::get_amount_of_denom(balance, cfg.denom.clone())?;

    if level != 0 && level != 1 {
        return Err(ContractError::InvalidBet {});
    }

    // Do flip   
    let obj = HashObj {
        time: env.block.time.seconds(),
        address: info.sender.clone(),
        level,
        count: cfg.flip_count
    };

    let hash = calculate_hash(&obj);
    
    let mut win = Some(1);
    
    if hash % 2 == 0 {
        win = Some(0);
    }
    
    let mut reward_amount = Uint128::zero();

    let owner_amount = amount * Uint128::from(constants::OWNER_RATE) / Uint128::from(constants::MULTIPLY);
    reward_amount = amount * Uint128::from(constants::REWARD_RATE) - owner_amount;

    let contract_amount = util::get_token_amount_of_address(deps.querier, cfg.denom.clone(), env.contract.address.clone())?;

    if contract_amount < reward_amount {
        win = Some(1);
    }

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), owner_amount, deps.api.addr_validate(constants::TREASURY_ADDR)?)?);

    match win {
        Some(0) => {
            //Player wins            
            messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), reward_amount, info.sender.clone())?);
        }
        Some(1) => {
            //Player Lose
        }
        _ => {
        }
    }

    let record = FHistory {
        id: cfg.flip_count + 1,
        address: info.sender.clone(),
        level,
        win,
        bet_amount: amount,
        timestamp: env.block.time.seconds()
    };
    FHISTORY.save(deps.storage, cfg.flip_count, &record)?;

    cfg.rps_count += 1;
    CONFIG.save(deps.storage, &cfg)?;
    
    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "flip"),
            attr("address", info.sender.clone()),
            attr("amount", amount),
            attr("win", win.expect("u8").to_string()),
        ]));
}

pub fn execute_rps(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    level: u64
) -> Result<Response, ContractError> {

    util::check_enabled(deps.storage)?;

    let mut cfg = CONFIG.load(deps.storage)?;

    let balance = Balance::from(info.funds);

    let amount = util::get_amount_of_denom(balance, cfg.denom.clone())?;

    if level != 0 && level != 1 && level != 2{
        return Err(ContractError::InvalidBet {});
    }

    // Do flip   
    let obj = HashObj {
        time: env.block.time.seconds(),
        address: info.sender.clone(),
        level,
        count: cfg.rps_count
    };

    let hash = calculate_hash(&obj);
    
    let cpu_move = hash % 3;
    
    let mut reward_amount = Uint128::zero();

    let owner_amount = amount * Uint128::from(constants::OWNER_RATE) / Uint128::from(constants::MULTIPLY);
    reward_amount = amount * Uint128::from(constants::REWARD_RATE) - owner_amount;

    let contract_amount = util::get_token_amount_of_address(deps.querier, cfg.denom.clone(), env.contract.address.clone())?;

    let mut win = get_winner(level, cpu_move);

    if contract_amount < reward_amount {
        win = Some(2);
    }

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), owner_amount, deps.api.addr_validate(constants::TREASURY_ADDR)?)?);

    match win {
        Some(0) => {
            //Player wins            
            messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), reward_amount, info.sender.clone())?);
        }
        Some(2) => {
            //Player Lose
        }
        _ => {
            // It's a tie
            reward_amount = amount - owner_amount;
            messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), reward_amount, info.sender.clone())?);
        }
    }

    let record = RHistory {
        id: cfg.rps_count + 1,
        address: info.sender.clone(),
        level,
        win,
        bet_amount: amount,
        timestamp: env.block.time.seconds()
    };
    RHISTORY.save(deps.storage, cfg.rps_count, &record)?;

    cfg.rps_count += 1;
    CONFIG.save(deps.storage, &cfg)?;
    
    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "rps"),
            attr("address", info.sender.clone()),
            attr("amount", amount),
            attr("win", win.expect("u8").to_string()),
        ]));
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
) -> Result<Response, ContractError> {

    util::check_owner(deps.storage, deps.api, info.sender.clone())?;

    let cfg = CONFIG.load(deps.storage)?;
    
    let contract_amount = util::get_token_amount_of_address(deps.querier, cfg.denom.clone(), env.contract.address.clone())?;

    if contract_amount < amount {
        return Err(ContractError::NotEnoughCoins {contract_amount });
    }

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(util::transfer_token_message(deps.querier, cfg.denom.clone(), amount, info.sender.clone())?);

    
    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "withdraw"),
            attr("address", info.sender.clone()),
            attr("amount", amount),
        ]));
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} 
            => to_binary(&query_config(deps, env)?),
        QueryMsg::RHistory {count} => to_binary(&query_rhistory(deps, count)?),
        QueryMsg::FHistory {count} => to_binary(&query_fhistory(deps, count)?),
    }
}

pub fn query_config(deps: Deps, env: Env) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    let treasury_amount = util::get_token_amount_of_address(deps.querier, cfg.denom.clone(), env.contract.address.clone()).unwrap();
    Ok(ConfigResponse {
        owner: cfg.owner,
        treasury_amount,
        denom: cfg.denom,
        enabled: cfg.enabled,
        flip_count: cfg.flip_count,
        rps_count: cfg.rps_count
    })
}

fn query_rhistory(
    deps: Deps,
    count: u32
) -> StdResult<RHistoryResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    let real_count = cfg.rps_count.min(count as u64) as usize;

    let mut list:Vec<RHistory> = vec![];
    for i in 0..real_count {
        list.push(RHISTORY.load(deps.storage, cfg.rps_count - 1 - i as u64)?);
    }
    
    Ok(RHistoryResponse {
        list
    })
    
}

fn query_fhistory(
    deps: Deps,
    count: u32
) -> StdResult<FHistoryResponse> {
    let cfg = CONFIG.load(deps.storage)?;

    let real_count = cfg.flip_count.min(count as u64) as usize;

    let mut list:Vec<FHistory> = vec![];
    for i in 0..real_count {
        list.push(FHISTORY.load(deps.storage, cfg.flip_count - 1 - i as u64)?);
    }
    
    Ok(FHistoryResponse {
        list
    })
    
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}

