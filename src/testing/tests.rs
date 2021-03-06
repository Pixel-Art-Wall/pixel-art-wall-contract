use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config_read, Color, Config, PixelExtension, PixelTokenInfo};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, coins, from_binary, Addr, Api, Deps, Response};
use cw721::{OwnerOfResponse, TokensResponse};
use cw721_base::state::TokenInfo;

const TEST_CREATOR: &str = "creator";
const TEST_MINT_FEE_AMOUNT: u128 = 2500000;
const TEST_USER: &str = "user";
const TEST_USER2: &str = "user2";
const TEST_TOKEN_ID1: u16 = 0;
const TEST_TOKEN_ID2: u16 = 1;
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

fn get_token_info(owner: Addr, colors: [[Color; 5]; 5], url: String) -> TokenInfo<PixelExtension> {
    TokenInfo::<PixelExtension> {
        owner,
        approvals: vec![],
        token_uri: None,
        extension: PixelExtension {
            pixel_colors: colors,
            url,
        },
    }
}

fn owner_of_query(deps: Deps, token_id: String) -> OwnerOfResponse {
    from_binary(
        &query(
            deps,
            mock_env(),
            QueryMsg::OwnerOf {
                token_id,
                include_expired: None,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

fn pixel_info_query(deps: Deps, token_id: String) -> PixelTokenInfo {
    from_binary(&query(deps, mock_env(), QueryMsg::PixelInfo { token_id }).unwrap()).unwrap()
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        Config {
            owner: deps.api.addr_canonicalize(TEST_CREATOR).unwrap(),
            mint_fee
        },
        config
    );
}

#[test]
fn can_mint_pixel() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let user = mock_info(TEST_USER, &[mint_fee.clone()]);

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: None,
        url: None,
    };
    // Mint first NFT.
    let res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "mint_pixel")
            .add_attribute("minter", user.sender.clone())
            .add_attribute("mint_fee", format!("{:?}", mint_fee))
            .add_attribute("token_id", TEST_TOKEN_ID1.to_string())
            .add_attribute("url", "")
            .add_attribute("color_map", format!("{:?}", EMPTY_COLORS)),
        res
    );

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID2,
        color_map: None,
        url: None,
    };

    // Mint second NFT.
    let res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "mint_pixel")
            .add_attribute("minter", user.sender.clone())
            .add_attribute("mint_fee", format!("{:?}", mint_fee))
            .add_attribute("token_id", TEST_TOKEN_ID2.to_string())
            .add_attribute("url", "")
            .add_attribute("color_map", format!("{:?}", EMPTY_COLORS)),
        res
    );

    // Verify that first NFT was correctly created.
    let actual_token_info: PixelTokenInfo =
        pixel_info_query(deps.as_ref(), TEST_TOKEN_ID1.to_string());
    let expected_token_info = get_token_info(user.sender.clone(), EMPTY_COLORS, "".to_string());
    assert_eq!(expected_token_info, actual_token_info);

    // Check that we can get the owner of the first NFT.
    let owner_response: OwnerOfResponse = owner_of_query(deps.as_ref(), TEST_TOKEN_ID1.to_string());
    assert_eq!(
        OwnerOfResponse {
            owner: user.sender.to_string(),
            approvals: vec![]
        },
        owner_response
    );

    // Verify that second NFT was correctly created.
    let actual_token_info: PixelTokenInfo =
        pixel_info_query(deps.as_ref(), TEST_TOKEN_ID2.to_string());
    assert_eq!(expected_token_info, actual_token_info);

    // Check that we can get the owner of the second NFT.
    let owner_response: OwnerOfResponse = owner_of_query(deps.as_ref(), TEST_TOKEN_ID2.to_string());
    assert_eq!(
        OwnerOfResponse {
            owner: user.sender.to_string(),
            approvals: vec![]
        },
        owner_response
    );

    // Check that we can get all tokens for the user.
    let query_res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Tokens {
            owner: user.sender.to_string(),
            start_after: None,
            limit: None,
        },
    )
    .unwrap();

    let tokens_response: TokensResponse = from_binary(&query_res).unwrap();
    assert_eq!(
        TokensResponse {
            tokens: vec![TEST_TOKEN_ID1.to_string(), TEST_TOKEN_ID2.to_string()]
        },
        tokens_response
    );
}

#[test]
fn can_not_mint_existing_pixel() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: None,
        url: None,
    };
    let user = mock_info(TEST_USER, &[mint_fee]);
    // First call is successful.
    let _res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg.clone()).unwrap();

    // Second call fails.
    let error = execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap_err();
    assert_eq!(ContractError::Claimed {}, error);
}

#[test]
fn can_not_mint_out_of_range_pixel() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let mint_msg = ExecuteMsg::Mint {
        token_id: 40_000,
        color_map: None,
        url: None,
    };
    let user = mock_info(TEST_USER, &[mint_fee]);
    let error = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap_err();
    assert_eq!(ContractError::InvalidTokenRange {}, error);
}

#[test]
fn can_mint_pixel_with_url_and_colors() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let user = mock_info(TEST_USER, &[mint_fee.clone()]);

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: Some(TEST_COLORS),
        url: Some(TEST_URL.to_string()),
    };

    let res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "mint_pixel")
            .add_attribute("minter", user.sender.clone())
            .add_attribute("mint_fee", format!("{:?}", mint_fee))
            .add_attribute("token_id", TEST_TOKEN_ID1.to_string())
            .add_attribute("url", TEST_URL)
            .add_attribute("color_map", format!("{:?}", TEST_COLORS)),
        res
    );

    let actual_token_info: PixelTokenInfo =
        pixel_info_query(deps.as_ref(), TEST_TOKEN_ID1.to_string());
    let expected_token_info =
        get_token_info(user.sender.clone(), TEST_COLORS, TEST_URL.to_string());
    assert_eq!(expected_token_info, actual_token_info);
}

#[test]
fn can_change_url() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let user = mock_info(TEST_USER, &[mint_fee.clone()]);

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: Some(TEST_COLORS),
        url: None,
    };

    let res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "mint_pixel")
            .add_attribute("minter", user.sender.clone())
            .add_attribute("mint_fee", format!("{:?}", mint_fee))
            .add_attribute("token_id", TEST_TOKEN_ID1.to_string())
            .add_attribute("url", "")
            .add_attribute("color_map", format!("{:?}", TEST_COLORS)),
        res
    );

    let actual_token_info: PixelTokenInfo =
        pixel_info_query(deps.as_ref(), TEST_TOKEN_ID1.to_string());
    let expected_token_info = get_token_info(user.sender.clone(), TEST_COLORS, "".to_string());
    assert_eq!(expected_token_info, actual_token_info);

    let change_url_msg = ExecuteMsg::ChangePixelData {
        token_id: TEST_TOKEN_ID1,
        color_map: None,
        url: Some(TEST_URL.to_string()),
    };

    let res = execute(deps.as_mut(), mock_env(), user.clone(), change_url_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "change pixel data")
            .add_attribute("token_id", TEST_TOKEN_ID1.to_string())
            .add_attribute("color_map", format!("{:?}", TEST_COLORS))
            .add_attribute("url", TEST_URL.to_string()),
        res
    );

    let actual_token_info: PixelTokenInfo =
        pixel_info_query(deps.as_ref(), TEST_TOKEN_ID1.to_string());
    let expected_token_info =
        get_token_info(user.sender.clone(), TEST_COLORS, TEST_URL.to_string());
    assert_eq!(expected_token_info, actual_token_info);
}

#[test]
fn cannot_change_url_unminted() {
    let mut deps = mock_dependencies(&[]);
    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let user = mock_info(TEST_USER, &[mint_fee]);

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: Some(TEST_COLORS),
        url: None,
    };

    let _res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    let change_url_msg = ExecuteMsg::ChangePixelData {
        token_id: TEST_TOKEN_ID2,
        color_map: None,
        url: Some(TEST_URL.to_string()),
    };

    let res = execute(deps.as_mut(), mock_env(), user.clone(), change_url_msg);

    assert_eq!(Err(ContractError::DoesNotExist {}), res);
}

#[test]
fn cannot_change_url_not_owned() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let user = mock_info(TEST_USER, &[mint_fee.clone()]);
    let user2 = mock_info(TEST_USER2, &[mint_fee]);

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: Some(TEST_COLORS),
        url: None,
    };

    let _res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    let change_url_msg = ExecuteMsg::ChangePixelData {
        token_id: TEST_TOKEN_ID1,
        color_map: None,
        url: Some(TEST_URL.to_string()),
    };

    let res = execute(deps.as_mut(), mock_env(), user2, change_url_msg);

    assert_eq!(Err(ContractError::Unauthorized {}), res);
}

#[test]
fn can_change_color() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let user = mock_info(TEST_USER, &[mint_fee.clone()]);

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: None,
        url: Some(TEST_URL.to_string()),
    };

    let res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "mint_pixel")
            .add_attribute("minter", user.sender.clone())
            .add_attribute("mint_fee", format!("{:?}", mint_fee))
            .add_attribute("token_id", TEST_TOKEN_ID1.to_string())
            .add_attribute("url", TEST_URL.to_string())
            .add_attribute("color_map", format!("{:?}", EMPTY_COLORS)),
        res
    );

    let actual_token_info: PixelTokenInfo =
        pixel_info_query(deps.as_ref(), TEST_TOKEN_ID1.to_string());
    let expected_token_info =
        get_token_info(user.sender.clone(), EMPTY_COLORS, TEST_URL.to_string());
    assert_eq!(expected_token_info, actual_token_info);

    let change_color_msg = ExecuteMsg::ChangePixelData {
        token_id: TEST_TOKEN_ID1,
        color_map: Some(TEST_COLORS),
        url: None,
    };

    let res = execute(deps.as_mut(), mock_env(), user.clone(), change_color_msg).unwrap();

    assert_eq!(
        Response::new()
            .add_attribute("action", "change pixel data")
            .add_attribute("token_id", TEST_TOKEN_ID1.to_string())
            .add_attribute("color_map", format!("{:?}", TEST_COLORS))
            .add_attribute("url", TEST_URL.to_string()),
        res
    );

    let actual_token_info: PixelTokenInfo =
        pixel_info_query(deps.as_ref(), TEST_TOKEN_ID1.to_string());
    let expected_token_info =
        get_token_info(user.sender.clone(), TEST_COLORS, TEST_URL.to_string());
    assert_eq!(expected_token_info, actual_token_info);
}

#[test]
fn cannot_change_color_unminted() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let user = mock_info(TEST_USER, &[mint_fee]);

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: None,
        url: Some(TEST_URL.to_string()),
    };

    let _res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    let change_color_msg = ExecuteMsg::ChangePixelData {
        token_id: TEST_TOKEN_ID2,
        color_map: Some(TEST_COLORS),
        url: None,
    };

    let res = execute(deps.as_mut(), mock_env(), user.clone(), change_color_msg);

    assert_eq!(Err(ContractError::DoesNotExist {}), res);
}

#[test]
fn cannot_change_color_not_owned() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let user = mock_info(TEST_USER, &[mint_fee.clone()]);
    let user2 = mock_info(TEST_USER2, &[mint_fee]);

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: None,
        url: Some(TEST_URL.to_string()),
    };

    let _res = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg).unwrap();

    let change_color_msg = ExecuteMsg::ChangePixelData {
        token_id: TEST_TOKEN_ID1,
        color_map: Some(TEST_COLORS),
        url: None,
    };

    let res = execute(deps.as_mut(), mock_env(), user2, change_color_msg);

    assert_eq!(Err(ContractError::Unauthorized {}), res);
}

#[test]
fn can_not_mint_pixel_with_insufficient_funds() {
    let mut deps = mock_dependencies(&[]);

    let mint_fee = coin(TEST_MINT_FEE_AMOUNT, "uusd");
    let msg = InstantiateMsg {
        mint_fee: mint_fee.clone(),
    };
    let info = mock_info(TEST_CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let mint_msg = ExecuteMsg::Mint {
        token_id: TEST_TOKEN_ID1,
        color_map: None,
        url: None,
    };

    // Can't mint with incorrect amount
    let user = mock_info(TEST_USER, &coins(TEST_MINT_FEE_AMOUNT - 1, "uusd"));
    let error = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg.clone()).unwrap_err();
    assert_eq!(ContractError::InsufficientFunds {}, error);

    // Can't mint with incorrect denom
    let user = mock_info(TEST_USER, &coins(TEST_MINT_FEE_AMOUNT, "usd"));
    let error = execute(deps.as_mut(), mock_env(), user.clone(), mint_msg.clone()).unwrap_err();
    assert_eq!(ContractError::InsufficientFunds {}, error);
}
