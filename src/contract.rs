use crate::msg::{GreetResp, InstantiateMsg, QueryMsg};
use crate::state::ADMINS;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut,Env,Empty, MessageInfo, Response, StdResult,
};
use cw_storey::CwStorage;
 
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
 
    Ok(Response::new())
}

pub fn execute(_deps: DepsMut, _env: Env, _info: MessageInfo, _msg: Empty) -> StdResult<Response> {
    unimplemented!()
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
                &InstantiateMsg { admins: vec![] },
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
 
    // ...
}