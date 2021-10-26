use cosmwasm_std::{CanonicalAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub nft_contract: CanonicalAddr,
}

pub fn config_store(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
/// Since all colours have range [0, 255] a u8 suffices.
pub struct NftData {
    /// Red part of the colour.
    pub r: u8,
    /// Green part of the colour.
    pub g: u8,
    /// Blue part of the colour.
    pub b: u8,
}

pub const DATA: Map<&str, NftData> = Map::new("data");
