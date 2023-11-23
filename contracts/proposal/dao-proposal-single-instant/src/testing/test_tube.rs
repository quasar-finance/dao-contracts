#[cfg(test)]
pub mod test_tube {
    use cosmwasm_std::Addr;
    use cosmwasm_std::Coin;
    use cosmwasm_std::CosmosMsg;
    use osmosis_test_tube::Account;
    use osmosis_test_tube::{Module, OsmosisTestApp, SigningAccount, Wasm};

    use crate::msg::SingleChoiceInstantProposeMsg;
    use crate::state::VoteSignature;
    // use cosmrs::bip32::secp256k1::ecdsa::signature::
    const INITIAL_BALANCE_AMOUNT: u128 = 340282366920938463463374607431768211455u128;

    pub fn default_init() -> (
        OsmosisTestApp,
        Addr,
        Addr,
        SigningAccount,
        Vec<SigningAccount>,
    ) {
        init_test_contract(
            "./test-tube-build/wasm32-unknown-unknown/release/cl_vault.wasm",
            10,
        )
    }

    pub fn init_test_contract(
        filename: &str,
        voters_number: u32,
    ) -> (
        OsmosisTestApp,
        Addr,
        Addr,
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

        let mut voters: Vec<SigningAccount> = vec![];
        for _ in 0..voters_number {
            voters.push(
                app.init_account(&[Coin::new(INITIAL_BALANCE_AMOUNT, "uosmo")])
                    .unwrap(),
            )
        }

        // TODO: Take store and instantiate steps from testing::instantiate::instantiate_with_cw4_groups_governance() unit test
        // TODO: Dao core
        // TODO: Proposal module
        // TODO: External dependencies
        // TODO: ...

        // Load compiled wasm bytecode
        let wasm_byte_code = std::fs::read(filename).unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, &admin)
            .unwrap()
            .data
            .code_id;

        // Instantiate dao contracts
        let core_contract = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {
                    // TODO: inst message
                },
                Some(admin.address().as_str()),
                Some("dao-dao-core"),
                vec![].as_ref(),
                &admin,
            )
            .unwrap();

        // Instantiate dao contracts
        let proposal_contract = wasm
            .instantiate(
                code_id,
                &InstantiateMsg {
                    // TODO: inst message
                },
                Some(admin.address().as_str()),
                Some("dao-proposal-single-instant"),
                vec![].as_ref(),
                &admin,
            )
            .unwrap();

        // TODO: Ensure we created memberships as:
        // - Proposers: admin, weight 0
        // - Voters: voters foreach, weight 1
        (
            app,
            Addr::unchecked(core_contract.data.address),
            Addr::unchecked(proposal_contract.data.address), // TODO: change this with proposal contract
            admin,
            voters,
        )
    }

    #[test]
    #[ignore]
    fn default_init_works() {
        let (app, _core_address, proposal_address, admin, voters) = default_init();
        let wasm = Wasm::new(&app);

        // TODO: Mock signatures taking voter.publickey to recover the sig
        let vote_signatures: Vec<VoteSignature> = vec![];
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
        let execute_propose_resp = wasm.execute(
            &proposal_address.to_string(),
            &SingleChoiceInstantProposeMsg {
                title: "Title".to_string(),
                description: "Description".to_string(),
                msgs: vec![], // TODO: Mock a simple bank transfer that in prod will be the trigger exec to the middleware contract
                proposer: Some(admin.address()),
                votes: vec![],
            },
            &vec![],
            &admin
        ).unwrap();

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
