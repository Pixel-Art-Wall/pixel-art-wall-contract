use crate::query as QueryHandler;
use cosmwasm_std::{
    from_binary, Addr, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw721::{ContractInfoResponse, OwnerOfResponse};
use cw721_base::{state::TokenInfo, Cw721Contract};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config_store, tokens, Color, Config, PixelExtension};

const PIXEL: &str = "pixel";

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    let cw721_contract = Cw721Contract::<PixelExtension, Empty>::default();

    let contract_info = ContractInfoResponse {
        name: PIXEL.to_string(),
        symbol: PIXEL.to_string(),
    };
    cw721_contract
        .contract_info
        .save(deps.storage, &contract_info)?;

    let config = Config {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        mint_fee: msg.mint_fee,
    };
    config_store(deps.storage).save(&config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

pub fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    position: u16,
    color_map: Option<[[Color; 5]; 5]>,
    url: Option<String>,
) -> Result<Response, ContractError> {
    let cw721_contract = Cw721Contract::<PixelExtension, Empty>::default();

    if position >= 40_000 {
        return Err(ContractError::InvalidTokenRange {});
    }

    if get_owner(deps.as_ref(), env, position).is_some() {
        return Err(ContractError::Claimed {});
    }

    check_sufficient_funds(info.funds, )

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

    let token_id = position.to_string();
    let token = TokenInfo::<PixelExtension> {
        owner: info.sender.clone(),
        approvals: vec![],
        token_uri: None,
        extension: PixelExtension {
            pixel_colors: new_color_map,
            url: new_url.clone(),
        },
    };
    tokens().update(deps.storage, &token_id, |old| match old {
        Some(_) => Err(ContractError::Claimed {}),
        None => Ok(token),
    })?;

    cw721_contract.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "mint_pixel")
        .add_attribute("minter", info.sender)
        .add_attribute("token_id", token_id)
        .add_attribute("url", new_url)
        .add_attribute("color_map", format!("{:?}", new_color_map)))
}

pub fn execute_change_pixel_data(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    position: u16,
    color_map: Option<[[Color; 5]; 5]>,
    url: Option<String>,
) -> Result<Response, ContractError> {
    if !token_minted(deps.as_ref(), env.clone(), position) {
        return Err(ContractError::DoesNotExist {});
    }

    let owner = get_owner(deps.as_ref(), env, position).unwrap();

    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let token_id = position.to_string();
    let token = tokens().load(deps.storage, &token_id)?;

    let extension = token.clone().extension;

    let new_url = match url {
        None => extension.url,
        Some(url) => url,
    };

    let new_color_map = match color_map {
        None => extension.pixel_colors,
        Some(color_map) => color_map,
    };

    let updated_extension = PixelExtension {
        pixel_colors: new_color_map,
        url: new_url.clone(),
    };

    let updated_token = TokenInfo::<PixelExtension> {
        owner: Addr::unchecked(owner),
        approvals: token.clone().approvals,
        token_uri: token.clone().token_uri,
        extension: updated_extension,
    };

    tokens().replace(deps.storage, &token_id, Some(&updated_token), Some(&token))?;

    Ok(Response::new()
        .add_attribute("action", "change pixel data")
        .add_attribute("token_id", token_id)
        .add_attribute("color_map", format!("{:?}", new_color_map))
        .add_attribute("url", new_url))
}

pub fn execute_update_config(
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

pub fn cw721_base_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let cw721_contract = Cw721Contract::<PixelExtension, Empty>::default();

    cw721_contract
        .execute(deps, env, info, msg.into())
        .map_err(|err| err.into())
}

fn token_minted(deps: Deps, env: Env, position: u16) -> bool {
    get_owner(deps, env, position).is_some()
}

fn get_owner(deps: Deps, env: Env, position: u16) -> Option<String> {
    let query_response = QueryHandler::cw721_base_query(
        deps,
        env,
        QueryMsg::OwnerOf {
            token_id: position.to_string(),
            include_expired: None,
        },
    );
    match query_response {
        Ok(result) => {
            let response: OwnerOfResponse = from_binary(&result).unwrap();
            Some(response.owner)
        }
        Err(_) => None,
    }
}

fn check_sufficient_funds(funds: Vec<Coin>, required: Coin) -> Result<(), ContractError> {
    if required.amount.u128() == 0 {
        return Ok(());
    }
    if funds.len() != 1 {
        return Err(ContractError::InsufficientFunds {});
    }
    let sent_sufficient_funds =
        funds[0].denom == required.denom && funds[0].amount.u128() == required.amount.u128();
    if sent_sufficient_funds {
        Ok(())
    } else {
        Err(ContractError::InsufficientFunds {})
    }
}
