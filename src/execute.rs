use crate::query as QueryHandler;
use cosmwasm_std::{from_binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult, Addr};
use cw721::{ContractInfoResponse, OwnerOfResponse};
use cw721_base::{state::TokenInfo, Cw721Contract};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config_store, tokens, Color, Config, PixelExtension};

const PIXEL: &str = "pixel";

pub fn instantiate(deps: DepsMut, info: MessageInfo, _msg: InstantiateMsg) -> StdResult<Response> {
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

pub fn execute_change_url(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    position: u16,
    url: String,
) -> Result<Response, ContractError> {

    let owner_check = get_owner(deps.as_ref(), env.clone(), position);
    let owner;

    if owner_check.is_some() {
        owner = owner_check.unwrap();
    } else {
        return Err(ContractError::DoesNotExist {});
    }


    if owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let token_id = position.to_string();
    let token = tokens().load(deps.storage, &token_id)?;

    let extension = token.extension;

    let updated_extension = PixelExtension {
        pixel_colors: extension.pixel_colors,
        url: url.clone(),
    };

    let updated_token = TokenInfo::<PixelExtension> {
        owner: Addr::unchecked(owner),
        approvals: token.approvals,
        token_uri: token.token_uri,
        extension: updated_extension
    };

    tokens().update(deps.storage, &token_id, |existing| match existing {
        None => {Err(ContractError::Unauthorized {})}
        Some(_) => {Ok(updated_token)}
    })?;

    Ok(Response::new()
        .add_attribute("action", "change url")
        .add_attribute("token_id", token_id)
        .add_attribute("url", url))
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
