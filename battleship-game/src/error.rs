use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid number of ships.")]
    InvalidShips {},

    #[error("Invalid board.")]
    InvalidBoard {},

    #[error("Wrong player to play.")]
    WrongTurn {},

    #[error("Player not found.")]
    PlayerNotFound {},

    #[error("Already sunk.")]
    AlreadySunk {},

    #[error("Game is over.")]
    GameFinished {},

}