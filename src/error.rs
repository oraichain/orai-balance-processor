use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Admin is none. Admin cannot be none!")]
    AdminEmpty {},
    #[error("Caller is not an admin. Cannot call this function")]
    InvalidAdmin {},
    #[error("Balance info of the given address already exists in the list. Cannot add more")]
    BalanceInfoExists {},
    #[error("Balance info of the given address does not exist. Cannot update")]
    BalanceInfoNotExist {},
    #[error("The balance mapping that you are trying to update does not exist. Cannot update")]
    BalanceMappingNotExist {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
