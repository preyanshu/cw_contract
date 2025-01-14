use crate::msg::{GreetResp, InstantiateMsg, QueryMsg , ExecuteMsg};
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut,Env,Empty, MessageInfo, Response,StdError, StdResult,
};
use cw_storey::CwStorage;
use crate::error::ContractError;
use crate::state::{ADMINS, DONATION_DENOM};
 

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admins = msg
        .admins
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr))
        .collect::<StdResult<Vec<_>>>()?;
 
    let mut cw_storage = CwStorage(deps.storage);
    ADMINS.access(&mut cw_storage).set(&admins)?;
    DONATION_DENOM
        .access(&mut cw_storage)
        .set(&msg.donation_denom)?;
 
    Ok(Response::new())
}


pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError>  {
    use ExecuteMsg::*;
 
    match msg {
        AddMembers { admins } => exec::add_members(deps, info, admins),
        Leave {} => exec::leave(deps, info),
        Donate {} => exec::donate(deps, info),
    }
}
 
mod exec {

    use cosmwasm_std::{coins, BankMsg};
    use super::*;
 
    pub fn add_members(
        deps: DepsMut,
        info: MessageInfo,
        admins: Vec<String>,
    ) -> Result<Response, ContractError> {
        let mut cw_storage = CwStorage(deps.storage);
 
        // Consider proper error handling instead of `unwrap`.
        let mut curr_admins = ADMINS.access(&cw_storage).get()?.unwrap();
        if !curr_admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {
                sender: info.sender,
            });
        }
 
        let admins: StdResult<Vec<_>> = admins
            .into_iter()
            .map(|addr| deps.api.addr_validate(&addr))
            .collect();
 
        curr_admins.append(&mut admins?);
        ADMINS.access(&mut cw_storage).set(&curr_admins)?;
 
        Ok(Response::new())
    }
 
    pub fn leave(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {

        let mut cw_storage = CwStorage(deps.storage);
 
        // Consider proper error handling instead of `unwrap`.
        let curr_admins = ADMINS.access(&cw_storage).get()?.unwrap();
  
        let admins: Vec<_> = curr_admins
            .into_iter()
            .filter(|admin| *admin != info.sender)
            .collect();
 
        ADMINS.access(&mut cw_storage).set(&admins)?;
 
        Ok(Response::new())
    }


    pub fn donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let cw_storage = CwStorage(deps.storage);
 
        let denom = DONATION_DENOM.access(&cw_storage).get()?.unwrap();
        let admins = ADMINS.access(&cw_storage).get()?.unwrap();
 
        let donation = cw_utils::must_pay(&info, &denom)?.u128();
 
        let donation_per_admin = donation / (admins.len() as u128);
 
        let messages = admins.into_iter().map(|admin| BankMsg::Send {
            to_address: admin.to_string(),
            amount: coins(donation_per_admin, &denom),
        });
 
        let resp = Response::new()
            .add_messages(messages)
            .add_attribute("action", "donate")
            .add_attribute("amount", donation.to_string())
            .add_attribute("per_admin", donation_per_admin.to_string());
 
        Ok(resp)
    }
}
 
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
 
    match msg {
        Greet {} => to_json_binary(&query::greet()?),
        AdminsList {} => to_json_binary(&query::admins_list(deps)?),
    }
}
 
mod query {
    use crate::msg::AdminsListResp;
 
    use super::*;
 
    pub fn greet() -> StdResult<GreetResp> {
        let resp = GreetResp {
            message: "Hello World".to_owned(),
        };
 
        Ok(resp)
    }
 
    pub fn admins_list(deps: Deps) -> StdResult<AdminsListResp> {
        let cw_storage = CwStorage(deps.storage);
        let admins = ADMINS.access(&cw_storage).get()?;
        let resp = AdminsListResp {
            admins: admins.unwrap_or_default(),
        };
        Ok(resp)
    }
}


 
#[cfg(test)]
mod tests {
    use cw_multi_test::{App, ContractWrapper, Executor, IntoAddr};
    use cosmwasm_std::coins;
 
    use crate::msg::AdminsListResp;
 
    use super::*;
 
    #[test]
    fn instantiation() {
        let mut app = App::default();
 
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let owner = "owner".into_addr();
        let admin1 = "admin1".into_addr();
        let admin2 = "admin2".into_addr();
 
        let addr = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &InstantiateMsg { admins: vec![] , donation_denom: "uscrt".to_string()},
                &[],
                "Contract",
                None,
            )
            .unwrap();
 
        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();
 
        assert_eq!(resp, AdminsListResp { admins: vec![] });
 
        let addr = app
            .instantiate_contract(
                code_id,
                owner,
                &InstantiateMsg {
                    admins: vec![admin1.to_string(), admin2.to_string()],
                    donation_denom: "uscrt".to_string(),
                },
                &[],
                "Contract 2",
                None,
            )
            .unwrap();
 
        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();
 
        assert_eq!(
            resp,
            AdminsListResp {
                admins: vec![admin1, admin2]
            }
        );
    }

    #[test]
    fn unauthorized() {
        let mut app = App::default();
 
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let owner = "owner".into_addr();
 
        let addr = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &InstantiateMsg { admins: vec![] , donation_denom: "uscrt".to_string()},
                &[],
                "Contract",
                None,
            )
            .unwrap();
 
        let err = app
            .execute_contract(
                owner.clone(),
                addr,
                &ExecuteMsg::AddMembers {
                    admins: vec!["user".to_owned()],
                },
                &[],
            )
            .unwrap_err();
 
        assert_eq!(
            ContractError::Unauthorized { sender: owner },
            err.downcast().unwrap()
        );
    }


    #[test]
    fn add_members() {
        let mut app = App::default();
 
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let owner = "owner".into_addr();
        let admin1 = "admin1".into_addr();
        let admin2 = "admin2".into_addr();
 
        let addr = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &InstantiateMsg { admins: vec![owner.to_string()] , donation_denom: "uscrt".to_string()},
                &[],
                "Contract",
                None,
            )
            .unwrap();
 
        app.execute_contract(
            owner.clone(),
            addr.clone(),
            &ExecuteMsg::AddMembers {
                admins: vec![admin1.to_string(), admin2.to_string()],
            },
            &[],
        )
        .unwrap();
 
        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();
 
        assert_eq!(
            resp,
            AdminsListResp {
                admins: vec![owner,admin1, admin2]
            }
        );
    }


    #[test]
    fn donations() {
        let owner = "owner".into_addr();
        let user = "user".into_addr();
        let admin1 = "admin1".into_addr();
        let admin2 = "admin2".into_addr();
 
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &user, coins(5, "eth"))
                .unwrap()
        });
 
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
 
        let addr = app
            .instantiate_contract(
                code_id,
                owner,
                &InstantiateMsg {
                    admins: vec![admin1.to_string(), admin2.to_string()],
                    donation_denom: "eth".to_owned(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();
 
        app.execute_contract(
            user.clone(),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(5, "eth"),
        )
        .unwrap();
 
        assert_eq!(
            app.wrap()
                .query_balance(user.as_str(), "eth")
                .unwrap()
                .amount
                .u128(),
            0
        );
 
        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            1
        );
 
        assert_eq!(
            app.wrap()
                .query_balance(admin1.as_str(), "eth")
                .unwrap()
                .amount
                .u128(),
            2
        );
 
        assert_eq!(
            app.wrap()
                .query_balance(admin2.as_str(), "eth")
                .unwrap()
                .amount
                .u128(),
            2
        );
    }
 
    // ...
}