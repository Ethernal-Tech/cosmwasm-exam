#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Deps, DepsMut, Env, MessageInfo, Response, Empty
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty
) -> Response {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty
) -> Response {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    _deps: Deps,
    _env: Env,
    _msg: Empty
) -> Response {
    unimplemented!();
}

mod execute {
    
}

mod query {

}

#[cfg(test)]
mod tests {

}