use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct PlayerInstantiate {
    pub address: String,
    pub stake: Uint128,
    pub board: Vec<Vec<bool>>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub ships: u128,
    pub players: Vec<PlayerInstantiate>,
}

#[cw_serde]
pub enum QueryMsg {
    GetAdmin {},
    GetPlayers {},
    GetTurn {},
    GetShips {},
}