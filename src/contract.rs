#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg, Addr, wasm_execute};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config_read, config_store, Config, DATA, NftData};
use cw721_base::{MintMsg, Extension};
use crate::error::ContractError::DoesNotExist;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nft-pixel-wall";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        nft_contract: CanonicalAddr::from(vec![]),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    config_store(deps.storage).save(&config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, info, owner),
        ExecuteMsg::RegisterContracts { nft_contract } => {
            register_contracts(deps, info, nft_contract)
        },
        ExecuteMsg::MintPixel {
            token_id,
        } => mint_pixel(deps, info, token_id),
        ExecuteMsg::ChangeRgb {
            token_id,
            r,
            g,
            b,
        } => change_rgb(deps, info, token_id, r, g, b),
    }
}

pub fn register_contracts(
    deps: DepsMut,
    info: MessageInfo,
    nft_contract: String,
) -> Result<Response, ContractError> {
    let mut config: Config = config_read(deps.storage).load()?;
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str())?
        || config.nft_contract != CanonicalAddr::from(vec![])
    {
        return Err(ContractError::Unauthorized {});
    }

    config.nft_contract = deps.api.addr_canonicalize(&nft_contract)?;
    config_store(deps.storage).save(&config)?;

    Ok(Response::default())
}

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
) -> Result<Response, ContractError> {
    let api = deps.api;
    config_store(deps.storage).update(|mut config| {
        if config.owner != api.addr_canonicalize(info.sender.as_str())? {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = api.addr_canonicalize(&owner)?;
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub fn mint_pixel(
    deps: DepsMut,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let mut config: Config = config_read(deps.storage).load()?;

    //if token exists, exit
    DATA.update(deps.storage, &token_id, |existing| match existing {
        None => Ok(NftData{
            r: 0,
            g: 0,
            b: 0
        }),
        Some(_) => Err(ContractError::Unauthorized {}),
    });

    let mint_msg = MintMsg::<Extension> {
        token_id,
        owner: String::from(info.sender),
        token_uri: None,
        extension: None,
    };

    res = wasm_execute(String::from(config.nft_contract), to_binary(&mint_msg), vec![]);




    Ok(Response::new())
}

pub fn change_rgb(
    deps: DepsMut,
    info: MessageInfo,
    token_id: String,
    r: u8,
    g: u8,
    b: u8,
) -> Result<Response, ContractError> {
    //todo: check if token exists
    //todo: check if token.owner = info.sender
    DATA.update(deps.storage, &token_id, |existing| match existing {
        None => Err(DoesNotExist {}),
        Some(_) => Ok(NftData{
            r,
            g,
            b,
        }),
    });
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = config_read(deps.storage).load()?;
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        nft_contract: deps.api.addr_humanize(&config.nft_contract)?.to_string(),
    })
}
