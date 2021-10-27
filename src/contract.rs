#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, wasm_execute, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::querier::query_token_owner;
use crate::state::{config_read, config_store, Color, Config, NftData, DATA};
use cw721_base::{Extension, MintMsg};

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
        ExecuteMsg::RegisterContract { nft_contract } => {
            register_contract(deps, info, nft_contract)
        }
        ExecuteMsg::MintPixel {
            token_id,
            color_map,
            url,
        } => mint_pixel(deps, info, token_id, color_map, url),
    }
}

pub fn register_contract(
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
    token_id: u16,
    color_map: Option<[[Color; 5]; 5]>,
    url: Option<String>,
) -> Result<Response, ContractError> {
    if token_id >= 40_000 {
        return Err(ContractError::InvalidTokenRange {});
    }

    let config: Config = config_read(deps.storage).load()?;
    let token_id = token_id.to_string();
    if query_token_owner(
        &deps.querier,
        deps.api.addr_humanize(&config.nft_contract)?,
        token_id.clone(),
    )
    .is_ok()
    {
        return Err(ContractError::TokenAlreadyMinted {});
    };

    let new_color_map = {
        if let Some(color_map) = color_map {
            color_map
        } else {
            [[Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }; 5]; 5]
        }
    };

    let new_url = {
        if let Some(url) = url {
            url
        } else {
            String::from("")
        }
    };

    DATA.update(deps.storage, &token_id, |existing| match existing {
        None => Ok(NftData {
            pixel_colors: new_color_map,
            url: new_url.clone(),
        }),
        Some(_) => Err(ContractError::TokenAlreadyMinted {}),
    })?;

    let mint_msg = MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from(&info.sender),
        token_uri: None,
        extension: None,
    };

    let _res = wasm_execute(
        deps.api.addr_humanize(&config.nft_contract)?,
        &mint_msg,
        vec![],
    )
    .unwrap();
    Ok(Response::new()
        .add_attribute("action", "mint_pixel")
        .add_attribute("minter", info.sender)
        .add_attribute("token_id", token_id)
        .add_attribute("url", new_url)
        .add_attribute("color_map", format!("{:?}", new_color_map)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::PixelData { token_id } => to_binary(&query_pixel_data(deps, token_id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = config_read(deps.storage).load()?;
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        nft_contract: deps.api.addr_humanize(&config.nft_contract)?.to_string(),
    })
}

fn query_pixel_data(deps: Deps, token_id: u16) -> StdResult<NftData> {
    DATA.load(deps.storage, &token_id.to_string())
}
