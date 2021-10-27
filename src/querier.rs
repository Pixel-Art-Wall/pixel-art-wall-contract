use cosmwasm_std::{to_binary, Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use cw721::OwnerOfResponse;
use cw721_base::msg::QueryMsg::NftInfo;

pub fn query_token_owner(
    querier: &QuerierWrapper,
    nft_address: Addr,
    token_id: String,
) -> StdResult<String> {
    let res: OwnerOfResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: nft_address.to_string(),
        msg: to_binary(&NftInfo { token_id })?,
    }))?;
    Ok(res.owner)
}
