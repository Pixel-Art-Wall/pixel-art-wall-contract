#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute as ExecHandler;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query as QueryHandler;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:nft-pixel-wall";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    ExecHandler::instantiate(deps, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { owner } => ExecHandler::execute_update_config(deps, info, owner),
        ExecuteMsg::Mint {
            token_id,
            color_map,
            url,
        } => ExecHandler::execute_mint(deps, info, env, token_id, color_map, url),
        ExecuteMsg::ChangeUrl {
            token_id,
            url
        } => ExecHandler::execute_change_url(deps, info, env, token_id, url),
        ExecuteMsg::ChangeColor {
            token_id,
            color_map,
        } => ExecHandler::execute_change_color(deps, info, env, token_id, color_map),
        _ => ExecHandler::cw721_base_execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&QueryHandler::query_config(deps)?),
        QueryMsg::PixelInfo { token_id } => {
            to_binary(&QueryHandler::query_pixel_nft_info(deps, token_id)?)
        }
        _ => QueryHandler::cw721_base_query(deps, env, msg),
    }
}
