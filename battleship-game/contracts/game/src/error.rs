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

    #[error("Invalid stake amount.")]
    InvalidStake {},

    #[error("Game has already started.")]
    GameStarted {},

    #[error("Unauthorized access.")]
    Unauthorized {},

    #[error("Game not started.")]
    GameNotStarted {},

    #[error("Overflow occurred during math operation.")]
    Overflow {},

    #[error("Turn not expired.")]
    TurnNotExpired {},

    #[error("Turn expired.")]
    TurnExpired {},

}

// impl From<OverflowError> for ContractError {
//     fn from(_: OverflowError) -> Self {
//         ContractError::Overflow {}
//     }
// }