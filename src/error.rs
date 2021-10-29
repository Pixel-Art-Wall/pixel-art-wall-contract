use cosmwasm_std::StdError;
use cw721_base::ContractError as CW721ContractError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Claimed")]
    Claimed {},

    #[error("Expired")]
    Expired {},

    #[error("Invalid token range")]
    InvalidTokenRange {},

    #[error("Token ID Does Not Exist")]
    DoesNotExist {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

impl From<CW721ContractError> for ContractError {
    fn from(msg: CW721ContractError) -> ContractError {
        match msg {
            CW721ContractError::Unauthorized {} => ContractError::Unauthorized {},
            CW721ContractError::Claimed {} => ContractError::Claimed {},
            CW721ContractError::Expired {} => ContractError::Expired {},
            CW721ContractError::Std(e) => ContractError::Std(e),
        }
    }
}
