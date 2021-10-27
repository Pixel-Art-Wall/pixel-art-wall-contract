use crate::contract::{execute, instantiate, query};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config_read, Color, Config, NftData};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, Api, CanonicalAddr, Response};

const TEST_CREATOR: &str = "creator";
const TEST_USER: &str = "user";
const TEST_TOKEN_ID: u16 = 0;
const TEST_URL: &str = "url";
const TEST_COLORS: [[Color; 5]; 5] = [[Color {
    r: 2,
    g: 2,
    b: 2,
    a: 2,
}; 5]; 5];
const EMPTY_COLORS: [[Color; 5]; 5] = [[Color {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
}; 5]; 5];
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

    let msg = ExecuteMsg::RegisterContract {
        nft_contract: NFT_CONTRACT.to_string(),
    };
    let user = mock_info(TEST_USER, &[]);
    // Non-creator can't register contract.
    let res = execute(deps.as_mut(), mock_env(), user, msg.clone());
    assert!(res.is_err());

    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

    // Can't register contract twice.
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    assert!(res.is_err());

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

#[test]
fn can_mint_pixel() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {};
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::RegisterContract {
        nft_contract: NFT_CONTRACT.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let mint_msg = ExecuteMsg::MintPixel {
        token_id: TEST_TOKEN_ID,
        color_map: None,
        url: None,
    };
    let user = mock_info(TEST_USER, &[]);
    let res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "mint_pixel")
            .add_attribute("minter", user.sender)
            .add_attribute("token_id", TEST_TOKEN_ID.to_string())
            .add_attribute("url", "")
            .add_attribute("color_map", format!("{:?}", EMPTY_COLORS)),
        res
    );

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PixelData {
            token_id: TEST_TOKEN_ID,
        },
    )
    .unwrap();
    let expected_nft_data: NftData = from_binary(&query_res).unwrap();
    assert_eq!(
        NftData {
            url: String::from(""),
            pixel_colors: EMPTY_COLORS
        },
        expected_nft_data
    );
}

#[test]
fn can_not_mint_existing_pixel() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {};
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::RegisterContract {
        nft_contract: NFT_CONTRACT.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let mint_msg = ExecuteMsg::MintPixel {
        token_id: TEST_TOKEN_ID,
        color_map: None,
        url: None,
    };
    // First call is successful.
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), mint_msg.clone()).unwrap();

    // Second call fails.
    let res = execute(deps.as_mut(), mock_env(), info, mint_msg);
    assert!(res.is_err());
}

#[test]
fn can_not_mint_out_of_range_pixel() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {};
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::RegisterContract {
        nft_contract: NFT_CONTRACT.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let mint_msg = ExecuteMsg::MintPixel {
        token_id: 40_000,
        color_map: None,
        url: None,
    };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), mint_msg.clone());
    assert!(res.is_err());
}

#[test]
fn can_mint_pixel_with_url_and_colors() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {};
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::RegisterContract {
        nft_contract: NFT_CONTRACT.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let mint_msg = ExecuteMsg::MintPixel {
        token_id: TEST_TOKEN_ID,
        color_map: Some(TEST_COLORS),
        url: Some(String::from(TEST_URL)),
    };
    let res = execute(deps.as_mut(), mock_env(), info.clone(), mint_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "mint_pixel")
            .add_attribute("minter", info.sender)
            .add_attribute("token_id", TEST_TOKEN_ID.to_string())
            .add_attribute("url", TEST_URL)
            .add_attribute("color_map", format!("{:?}", TEST_COLORS)),
        res
    );

    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::PixelData {
            token_id: TEST_TOKEN_ID,
        },
    )
    .unwrap();
    let expected_nft_data: NftData = from_binary(&query_res).unwrap();
    assert_eq!(
        NftData {
            url: String::from(TEST_URL),
            pixel_colors: TEST_COLORS
        },
        expected_nft_data
    );
}
