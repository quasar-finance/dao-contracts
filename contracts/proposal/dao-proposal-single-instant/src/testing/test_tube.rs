#[cfg(test)]
pub mod test_tube {
    use std::collections::HashMap;

    use cosmwasm_std::{Coin, Uint128};
    use cw_utils::Duration;
    use dao_interface::msg::InstantiateMsg as InstantiateMsgCore;
    use dao_interface::state::ModuleInstantiateInfo;
    use dao_voting::pre_propose::PreProposeInfo;
    use dao_voting::threshold::Threshold;
    use osmosis_test_tube::Account;
    use osmosis_test_tube::{Module, OsmosisTestApp, SigningAccount, Wasm};

    use crate::msg::{
        InstantiateMsg as InstantiateMsgSingleChoiceInstant, SingleChoiceInstantProposeMsg,
    };
    use crate::state::VoteSignature;

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

        // Contracts to store and instantiate
        let contracts_setup: Vec<(&str, &str)> = vec![
            (
                "dao-voting",
                "./test-tube-build/wasm32-unknown-unknown/release/dao_voting.wasm", // TODO
            ),
            (
                "dao-proposal-single-instant",
                "./test-tube-build/wasm32-unknown-unknown/release/dao_proposal_single_instant.wasm", // TODO
            ),
            (
                "dao-dao-core",
                "./test-tube-build/wasm32-unknown-unknown/release/dao_dao_core.wasm", // TODO
            ),
        ];

        // Store contracts and declare a HashMap
        let code_ids: HashMap<&str, u64> = contracts_setup
            .iter()
            .map(|&(contract_name, file_name)| {
                let wasm_byte_code = std::fs::read(file_name)
                    .expect(format!("Failed to read file: {}", file_name).as_str());

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

        // TODO: START ISNTANTIATION
        // Voting
        let dao_voting_contract = wasm
            .instantiate(
                *code_ids.get("dao-voting").unwrap(),
                &todo!(),
                Some(admin.address().as_str()),
                Some("dao-voting"),
                vec![].as_ref(),
                &admin,
            )
            .unwrap();
        contracts.insert("dao-voting", dao_voting_contract.data.address.as_str());

        // Proposal
        let dao_proposal_contract = wasm
            .instantiate(
                *code_ids.get("dao-proposal-single-instant").unwrap(),
                &InstantiateMsgSingleChoiceInstant {
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
                },
                Some(admin.address().as_str()),
                Some("dao-proposal-single-instant"),
                vec![].as_ref(),
                &admin,
            )
            .unwrap();
        contracts.insert(
            "dao-proposal-single-instant",
            dao_proposal_contract.data.address.as_str(),
        );

        // Core
        let dao_dao_core = wasm
            .instantiate(
                *code_ids.get("dao-dao-core").unwrap(),
                &InstantiateMsgCore {
                    admin: Some(admin.address()),
                    name: "DAO DAO Core".to_string(),
                    description: "".to_string(),
                    image_url: None,
                    automatically_add_cw20s: true,
                    automatically_add_cw721s: true,
                    voting_module_instantiate_info: ModuleInstantiateInfo {
                        code_id: *code_ids.get("dao-voting").unwrap(),
                        msg: todo!(),
                        admin: Some(admin.address()),
                        funds: vec![],
                        label: "dao-voting".to_string(),
                    },
                    proposal_modules_instantiate_info: vec![ModuleInstantiateInfo {
                        code_id: *code_ids.get("dao-voting").unwrap(),
                        msg: todo!(),
                        admin: Some(admin.address()),
                        funds: vec![],
                        label: "dao-proposal-single-instant".to_string(),
                    }],
                    initial_items: None,
                    dao_uri: None,
                },
                Some(admin.address().as_str()),
                Some("dao-dao-core"),
                vec![].as_ref(),
                &admin,
            )
            .unwrap();
        contracts.insert("dao-dao-core", dao_dao_core.data.address.as_str());

        // TODO: END INSTANTIATION

        // TODO: Ensure memberships are created as specified
        // For example:
        // - Proposers: admin, weight 0
        // - Voters: voters foreach, weight 1

        (app, contracts, admin, voters)
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
                message_hash: msg,
                signature: voter.signing_key().sign(msg).unwrap().as_ref(),
            })
        }

        // Execute execute_propose (proposal, voting and execution in one single workflow)
        let execute_propose_resp = wasm
            .execute(
                contracts.get("dao-proposal-single-instant").unwrap(),
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

        // TODO: Query things from contract to make assertions
        // let resp = wasm
        //     .query::<QueryMsg, PoolResponse>(
        //         contract_address.as_str(),
        //         &QueryMsg::VaultExtension(ExtensionQueryMsg::ConcentratedLiquidity(
        //             ClQueryMsg::Pool {},
        //         )),
        //     )
        //     .unwrap();
    }
}
