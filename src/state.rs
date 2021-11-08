use cosmwasm_std::{Addr, CanonicalAddr, Coin, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use cw721_base::state::TokenInfo;
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub mint_fee: Coin,
}

pub fn config_store(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Copy)]
/// Since all colours have range [0, 255] a u8 suffices.
pub struct Color {
    /// Red part of the colour.
    pub r: u8,
    /// Green part of the colour.
    pub g: u8,
    /// Blue part of the colour.
    pub b: u8,
    /// free alpha
    pub a: u8,
}

// wrap an array of colours and a url to make up for the 5x5 pixel square
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PixelExtension {
    pub pixel_colors: [[Color; 5]; 5],
    pub url: String,
}

pub type PixelTokenInfo = TokenInfo<PixelExtension>;

pub struct TokenIndexes<'a> {
    pub owner: MultiIndex<'a, (Addr, Vec<u8>), PixelTokenInfo>,
}

impl<'a> IndexList<PixelTokenInfo> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PixelTokenInfo>> + '_> {
        let v: Vec<&dyn Index<PixelTokenInfo>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn tokens<'a>() -> IndexedMap<'a, &'a str, PixelTokenInfo, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
        owner: MultiIndex::new(
            |d: &PixelTokenInfo, k: Vec<u8>| (d.owner.clone(), k),
            "tokens",
            "tokens__owner",
        ),
    };
    IndexedMap::new("tokens", indexes)
}
