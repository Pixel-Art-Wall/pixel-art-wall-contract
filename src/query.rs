use cosmwasm_std::{Binary, Deps, Empty, Env, StdResult};
use cw721_base::Cw721Contract;

use crate::msg::QueryMsg;
use crate::state::{config_read, tokens, Config, PixelExtension, PixelTokenInfo};

pub fn query_config(deps: Deps) -> StdResult<Config> {
    config_read(deps.storage).load()
}

pub fn query_pixel_nft_info(deps: Deps, token_id: String) -> StdResult<PixelTokenInfo> {
    let token = tokens().load(deps.storage, &token_id)?;
    Ok(token)
}

pub fn cw721_base_query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let cw721_contract = Cw721Contract::<PixelExtension, Empty>::default();
    cw721_contract.query(deps, env, msg.into())
}
