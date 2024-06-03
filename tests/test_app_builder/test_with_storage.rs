use crate::test_contracts;
use crate::test_contracts::counter::{ CounterQueryMsg, CounterResponseMsg };
use cosmwasm_std::{ to_json_binary, Empty, Order, Record, Storage, WasmMsg, Uint128, coins };
use cw_multi_test::{ AppBuilder, Executor, StorageSnapshot, App, BankSudo, SudoMsg as CwSudoMsg };
use std::collections::BTreeMap;
use std::iter;


#[derive(Default)]
struct MyStorage(BTreeMap<Vec<u8>, Vec<u8>>);

// Minimal implementation of custom storage.
impl Storage for MyStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.get::<Vec<u8>>(&key.into()).cloned()
    }

    fn range<'a>(
        &'a self,
        _start: Option<&[u8]>,
        _end: Option<&[u8]>,
        _order: Order
    ) -> Box<dyn Iterator<Item = Record> + 'a> {
        Box::new(iter::empty())
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        self.0.insert(key.into(), value.into());
    }

    fn remove(&mut self, key: &[u8]) {
        self.0.remove(key);
    }
}

#[test]
fn building_app_with_custom_storage_should_work() {
    // prepare additional test input data
    let msg = to_json_binary(&(Empty {})).unwrap();
    let admin = None;
    let funds = vec![];
    let label = "my-counter";

    // build the application with custom storage
    let _app_builder = AppBuilder::default();
    //let mut app = app_builder.with_storage(MyStorage::default()).build(no_init);
    let mut app = App::default();
    // prepare user addresses
    let owner_addr = app.api().addr_make("owner");
    let user = app.api().addr_make("user");

    // store a contract code
    let code_id = app.store_code(test_contracts::counter::contract());

    // instantiate contract, this initializes a counter with value 1
    let contract_addr = app
        .instantiate_contract(
            code_id,
            owner_addr.clone(),
            &(WasmMsg::Instantiate {
                admin: admin.clone(),
                code_id,
                msg: msg.clone(),
                funds: funds.clone(),
                label: label.into(),
            }),
            &funds,
            label,
            admin
        )
        .unwrap();

    // execute contract, this increments a counter
    app.execute_contract(
        owner_addr.clone(),
        contract_addr.clone(),
        &(WasmMsg::Execute {
            contract_addr: contract_addr.clone().into(),
            msg: msg.clone(),
            funds: funds.clone(),
        }),
        &[]
    ).unwrap();

    // query contract for current counter value
    let response: CounterResponseMsg = app
        .wrap()
        .query_wasm_smart(&contract_addr, &(CounterQueryMsg::Counter {}))
        .unwrap();

    let fee_funds = coins(100000, "btc");
    //Mint native token for user
    app.sudo(
        CwSudoMsg::Bank({ BankSudo::Mint {
                to_address: user.to_string(),
                amount: fee_funds.clone(),
            } })
    )
        .map_err(|err| println!("{:?}", err))
        .ok();
    let balance_native_receipient = app.wrap().query_balance(&user.clone(), "btc").unwrap().amount;
    assert_eq!(Uint128::new(100000), balance_native_receipient);
    // counter should be 2
    assert_eq!(2, response.value);
    //Create a snaposhot
    let mut snapshot = StorageSnapshot::default();
    let codeid: u128 = 1;

    app.snapshot_storage(&mut snapshot, codeid).unwrap();
    //execute contract, this increments a counter
    app.execute_contract(
        owner_addr,
        contract_addr.clone(),
        &(WasmMsg::Execute {
            contract_addr: contract_addr.clone().into(),
            msg,
            funds,
        }),
        &[]
    ).unwrap();

    app.load_snapshot(&snapshot, codeid).unwrap();

    // query contract for current counter value
    let response: CounterResponseMsg = app
        .wrap()
        .query_wasm_smart(&contract_addr, &(CounterQueryMsg::Counter {}))
        .unwrap();

    // counter should be 2
    assert_eq!(2, response.value);
    app.sudo(
        CwSudoMsg::Bank({ BankSudo::Mint {
                to_address: user.to_string(),
                amount: fee_funds.clone(),
            } })
    )
        .map_err(|err| println!("{:?}", err))
        .ok();
    let balance_native_receipient = app.wrap().query_balance(&user, "btc").unwrap().amount;
    assert_eq!(Uint128::new(200000), balance_native_receipient);
    app.load_snapshot(&snapshot, codeid).unwrap();
    let balance_native_receipient = app.wrap().query_balance(&user, "btc").unwrap().amount;
    assert_eq!(Uint128::new(100000), balance_native_receipient);
}
