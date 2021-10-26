use crate::contract::{execute, instantiate, query};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config_read, Config};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Api, CanonicalAddr};

const TEST_CREATOR: &str = "creator";
const NFT_CONTRACT: &str = "nft_contract";

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {};
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        Config {
            owner: deps.api.addr_canonicalize(TEST_CREATOR).unwrap(),
            nft_contract: CanonicalAddr::from(vec![]),
        },
        config
    );

    let msg = ExecuteMsg::RegisterContracts {
        nft_contract: NFT_CONTRACT.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Query the config.
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let value: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        ConfigResponse {
            owner: TEST_CREATOR.to_string(),
            nft_contract: NFT_CONTRACT.to_string()
        },
        value
    );
}
