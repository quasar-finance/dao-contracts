#[cfg(test)]
pub mod test_tube {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use cosmwasm_std::{Coin, to_binary, Uint128};
    use cw_utils::Duration;
    use dao_interface::state::Admin;
    use dao_interface::msg::InstantiateMsg as InstantiateMsgCore;
    use dao_interface::state::ModuleInstantiateInfo;
    use dao_voting::pre_propose::PreProposeInfo;
    use dao_voting::threshold::Threshold;
    use dao_voting_cw4::msg::GroupContract;
    use crate::msg::{SingleChoiceInstantProposeMsg, InstantiateMsg};
    use crate::state::VoteSignature;
    use osmosis_test_tube::Account;
    use osmosis_test_tube::{Module, OsmosisTestApp, SigningAccount, Wasm};

    const INITIAL_BALANCE_AMOUNT: u128 = 340282366920938463463374607431768211455u128;

    pub fn test_init(
        voters_number: u32,
    ) -> (
        OsmosisTestApp,
        HashMap<&'static str, &'static str>,
        SigningAccount,
        Vec<SigningAccount>,
    ) {
        // Create new osmosis appchain instance
        let app = OsmosisTestApp::new();
        let wasm = Wasm::new(&app);

        // Create new account with initial funds
        let admin: SigningAccount = app
            .init_account(&[Coin::new(INITIAL_BALANCE_AMOUNT, "uosmo")])
            .unwrap();

        // Create voters accounts
        let mut voters: Vec<SigningAccount> = vec![];
        for _ in 0..voters_number {
            voters.push(
                app.init_account(&[Coin::new(INITIAL_BALANCE_AMOUNT, "uosmo")])
                    .unwrap(),
            )
        }

        // TODO: Check if more contracts are required to be isntantiated to have a minimal working environment for our purpose

        // Contracts to store and instantiate
        let contracts_setup: Vec<(&str, Vec<u8>)> = vec![
            // ./packages/dao-voting
            (
                "dao_voting",
                get_wasm_byte_code("dao_voting_cw4") // TODO: Check testing::instantiate::instantiate_with_cw4_groups_governance()
            ),
            // ./contracts/voting/dao-voting-cw4
            (
                "dao_voting_cw4",
                get_wasm_byte_code("dao_voting_cw4") // TODO: Check testing::instantiate::instantiate_with_cw4_groups_governance()
            ),
            // ./contracts/proposal/dao-proposal-single-instant
            (
                "dao_proposal_single_instant",
                get_wasm_byte_code("dao_proposal_single_instant")
            ),
            // ./contracts/dao-dao-core
            (
                "dao_dao_core",
                get_wasm_byte_code("dao_dao_core")
            ),
        ];

        // Store contracts and declare a HashMap
        let code_ids: HashMap<&str, u64> = contracts_setup
            .iter()
            .map(|&(contract_name, ref wasm_byte_code)| {
                let code_id = wasm
                    .store_code(&wasm_byte_code, None, &admin)
                    .expect("Failed to store code")
                    .data
                    .code_id;

                (contract_name, code_id)
            })
            .collect();

        // HashMap to store contract names and their addresses
        let mut contracts: HashMap<&str, &str> = HashMap::new();

        // TODO: Create msgs as defined here -> https://github.com/DA0-DA0/dao-contracts/wiki/Instantiating-a-DAO#proposal-module-instantiate-message
        // We should use structs and serde to serialize it to json, and then to base64

        let initial_members = voters.iter().map(|voter| cw4::Member {
            addr: voter.address().to_string(),
            weight: 1,
        }).collect::<Vec<_>>();

        // Core
        let dao_dao_core_instantiate_resp = wasm
            .instantiate(
                *code_ids.get("dao_dao_core").unwrap(),
                &InstantiateMsgCore {
                    admin: Some(admin.address()),
                    name: "DAO DAO Core".to_string(),
                    description: "".to_string(),
                    image_url: None,
                    automatically_add_cw20s: true,
                    automatically_add_cw721s: true,
                    voting_module_instantiate_info: ModuleInstantiateInfo {
                        code_id: *code_ids.get("dao_voting").unwrap(),
                        msg: to_binary(&dao_voting_cw4::msg::InstantiateMsg {
                            group_contract: GroupContract::New {
                                cw4_group_code_id: *code_ids.get("dao_voting_cw4").unwrap(),
                                initial_members,
                            },
                        })
                        .unwrap(),
                        admin: Some(Admin::CoreModule {}),
                        funds: vec![],
                        label: "DAO DAO voting module".to_string(),
                    },
                    proposal_modules_instantiate_info: vec![ModuleInstantiateInfo {
                        code_id: *code_ids.get("dao_proposal_single_instant").unwrap(),
                        msg: to_binary(&InstantiateMsg {
                            threshold: Threshold::AbsoluteCount {
                                threshold: Uint128::new(1u128),
                            },
                            // TODO: Create an additional test variant as below
                            // threshold: Threshold::ThresholdQuorum {
                            //     threshold: PercentageThreshold,
                            //     quorum: PercentageThreshold,
                            // },
                            max_voting_period: Duration::Time(0), // 0 seconds
                            min_voting_period: None,
                            only_members_execute: true,
                            allow_revoting: false,
                            pre_propose_info: PreProposeInfo::AnyoneMayPropose {},
                            close_proposal_on_execution_failure: true,
                        }).unwrap(),
                        admin: Some(Admin::CoreModule {}),
                        funds: vec![],
                        label: "DAO DAO governance module".to_string(),
                    }],
                    initial_items: None,
                    dao_uri: None,
                },
                Some(admin.address().as_str()),
                Some("dao_dao_core"),
                vec![].as_ref(),
                &admin,
            )
            .unwrap();

        // contracts.insert("dao_dao_core", dao_dao_core.data.address.as_str());
        println!("dao_dao_core_instantiate_resp: {:?}", dao_dao_core_instantiate_resp);

        (app, contracts, admin, voters)
    }

    fn get_wasm_byte_code(filename: &str) -> Vec<u8> {
        let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let byte_code = std::fs::read(
            manifest_path
                .join("..")
                .join("..")
                .join("..")
                .join("artifacts")
                .join(format!("{}-aarch64.wasm", filename)),
        );
        match byte_code {
            Ok(byte_code) => byte_code,
            // On arm processors, the above path is not found, so we try the following path
            Err(_) => std::fs::read(
                manifest_path
                    .join("..")
                    .join("..")
                    .join("..")
                    .join("artifacts")
                    .join(format!("{}-aarch64.wasm", filename)),
            )
            .unwrap(),
        }
    }

    #[test]
    #[ignore]
    fn test_dao_proposal_single_instant() {
        let (app, contracts, admin, voters) = test_init(3);
        let wasm = Wasm::new(&app);

        // TODO: Mock signatures taking voter.publickey to recover the sig
        let mut vote_signatures: Vec<VoteSignature> = vec![];
        for voter in voters {
            // Dummy message
            let msg: &[u8] = "Hello World!".as_bytes();
            // VoteSignature
            vote_signatures.push(VoteSignature {
                message_hash: msg.to_vec(),
                signature: voter.signing_key().sign(msg).unwrap().as_ref().to_vec(),
            })
        }

        // Execute execute_propose (proposal, voting and execution in one single workflow)
        let execute_propose_resp = wasm
            .execute(
                contracts.get("dao_proposal_single_instant").unwrap(),
                &SingleChoiceInstantProposeMsg {
                    title: "Title".to_string(),
                    description: "Description".to_string(),
                    msgs: vec![], // TODO: Mock a simple bank transfer that in prod will be the trigger exec to the middleware contract
                    proposer: Some(admin.address()),
                    votes: vec![],
                },
                &vec![],
                &admin,
            )
            .unwrap();

        println!("execute_propose_resp: {:?}", execute_propose_resp);
    }
}
