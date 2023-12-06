#[cfg(test)]

pub mod test_tube {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use cosmrs::bip32::PublicKey;
    // For message hex
    use crate::msg::{ExecuteMsg, InstantiateMsg, SingleChoiceInstantProposeMsg};
    use crate::state::VoteSignature;
    use cosmwasm_std::{to_binary, Coin, Uint128, Deps, DepsMut};
    use cw_utils::Duration;
    use dao_interface::msg::InstantiateMsg as InstantiateMsgCore;
    use dao_interface::state::Admin;
    use dao_interface::state::ModuleInstantiateInfo;
    use dao_voting::pre_propose::PreProposeInfo;
    use dao_voting::threshold::Threshold;
    use dao_voting_cw4::msg::GroupContract;
    use hex;
    use osmosis_test_tube::Account;
    use osmosis_test_tube::{Module, OsmosisTestApp, SigningAccount, Wasm};
    use sha2::{Digest, Sha256};

    /// Init constants
    const SLUG_DAO_DAO_CORE: &str = "dao_dao_core";
    const SLUG_CW4_GROUP: &str = "cw4_group";
    const SLUG_DAO_VOTING_CW4: &str = "dao_voting_cw4";
    const SLUG_DAO_PROPOSAL_SINGLE_INSTANT: &str = "dao_proposal_single_instant";

    /// Test constants
    const INITIAL_BALANCE_AMOUNT: u128 = 340282366920938463463374607431768211455u128;

    /*
    pub fn compute_sha256_hash(hex_message: &str) -> Result<String, hex::FromHexError> {
        let message_bytes = hex::decode(hex_message)?;
        let mut hasher = Sha256::new();
        hasher.update(&message_bytes);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
    */
    fn compute_sha256_hash(message: &str) -> Result<String, hex::FromHexError> {
        let message_bytes = hex::decode(message).expect("Invalid hex string");
        let mut hasher = Sha256::new();
        // hasher.update(message.as_bytes());
        hasher.update(message_bytes);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    pub fn test_init(
        voters_number: u32,
    ) -> (
        OsmosisTestApp,
        HashMap<&'static str, String>,
        SigningAccount,
        Vec<SigningAccount>,
    ) {
        // Create new osmosis appchain instance
        let app = OsmosisTestApp::new();
        let wasm = Wasm::new(&app);

        // Create new admin account with initial funds
        // The contract admin, to be used during store code.
        let admin: SigningAccount = app
            .init_account(&[Coin::new(INITIAL_BALANCE_AMOUNT, "uosmo")])
            .unwrap();

        // Create voters accounts with initial funds
        let mut voters: Vec<SigningAccount> = vec![];
        for _ in 0..voters_number {
            voters.push(
                app.init_account(&[Coin::new(INITIAL_BALANCE_AMOUNT, "uosmo")])
                    .unwrap(),
            )
        }

        for voter in &voters {
            println!("Prefix: {:?}", voter.prefix());
            println!("Signing Key: {:?} \n public key bytes - {:?} \n public key json {:?} \n public key URL{:?}",
                     voter.signing_key().public_key(),
                     voter.signing_key().public_key().to_bytes(),
                     voter.signing_key().public_key().to_json(),
                     voter.signing_key().public_key().type_url(),
            );
            println!("Fee Setting: {:?}", voter.fee_setting());
            println!("Public_key : {:?}", voter.public_key());
            println!("---------------------------");
        }

        // Create a vector of cw4::Member
        let initial_members = voters
            .iter()
            .map(|voter| cw4::Member {
                addr: voter.address().to_string(),
                weight: 1,
            })
            .collect::<Vec<_>>();
        // TODO: Consider admin should be included ^ as member, but with voting power (weight) 0
        println!("initial_members: {:?}", initial_members);
        // Contracts to store and instantiate
        // TODO: Check testing::instantiate::instantiate_with_cw4_groups_governance()
        let contracts_setup: Vec<(&str, Vec<u8>)> = vec![
            (
                SLUG_CW4_GROUP,
                get_wasm_byte_code(SLUG_CW4_GROUP), // this is copy pasted from outside as this workspace if not creating this artifact. it has been taken from https://github.com/CosmWasm/cw-plus/tree/v1.1.0
            ),
            (SLUG_DAO_VOTING_CW4, get_wasm_byte_code(SLUG_DAO_VOTING_CW4)),
            (
                SLUG_DAO_PROPOSAL_SINGLE_INSTANT,
                get_wasm_byte_code(SLUG_DAO_PROPOSAL_SINGLE_INSTANT),
            ),
            (SLUG_DAO_DAO_CORE, get_wasm_byte_code(SLUG_DAO_DAO_CORE)),
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
        println!("code_ids: {:?}", code_ids);
        // TODO: Instantiate msgs defined here -> https://github.com/DA0-DA0/dao-contracts/wiki/Instantiating-a-DAO#proposal-module-instantiate-message

        // Instantiate contract and sub-contracts in once
        let dao_dao_core_instantiate_resp = wasm
            .instantiate(
                *code_ids.get(SLUG_DAO_DAO_CORE).unwrap(),
                &InstantiateMsgCore {
                    admin: Some(admin.address()),
                    name: "DAO DAO Core".to_string(),
                    description: "".to_string(),
                    image_url: None,
                    automatically_add_cw20s: true,
                    automatically_add_cw721s: true,
                    voting_module_instantiate_info: ModuleInstantiateInfo {
                        code_id: *code_ids.get(SLUG_DAO_VOTING_CW4).unwrap(),
                        msg: to_binary(&dao_voting_cw4::msg::InstantiateMsg {
                            group_contract: GroupContract::New {
                                cw4_group_code_id: *code_ids.get(SLUG_CW4_GROUP).unwrap(),
                                initial_members,
                            },
                        })
                        .unwrap(),
                        admin: Some(Admin::CoreModule {}),
                        funds: vec![],
                        label: "DAO DAO voting module".to_string(),
                    },
                    proposal_modules_instantiate_info: vec![ModuleInstantiateInfo {
                        code_id: *code_ids.get(SLUG_DAO_PROPOSAL_SINGLE_INSTANT).unwrap(),
                        msg: to_binary(&InstantiateMsg {
                            threshold: Threshold::AbsoluteCount {
                                threshold: Uint128::new(1u128),
                            },
                            // TODO: Create an additional test variant as below
                            // threshold: Threshold::ThresholdQuorum {
                            //     threshold: PercentageThreshold,
                            //     quorum: PercentageThreshold,
                            // },
                            // max_voting_period: Duration::Time(0), // 0 seconds
                            max_voting_period: Duration::Height(0), // 0 blocks
                            min_voting_period: None,
                            only_members_execute: false, // TODO
                            allow_revoting: false,
                            pre_propose_info: PreProposeInfo::AnyoneMayPropose {}, // TODO
                            close_proposal_on_execution_failure: true,
                        })
                        .unwrap(),
                        admin: Some(Admin::CoreModule {}),
                        funds: vec![],
                        label: "DAO DAO governance module".to_string(),
                    }],
                    initial_items: None,
                    dao_uri: None,
                },
                Some(admin.address().as_str()),
                Some(SLUG_DAO_DAO_CORE),
                vec![].as_ref(),
                &admin,
            )
            .unwrap();
        println!("dao_dao_core_instantiate_resp: {:?}", dao_dao_core_instantiate_resp);
        // HashMap to store contract names and their addresses
        let mut contracts: HashMap<&str, String> = HashMap::new();

        for event in dao_dao_core_instantiate_resp.events {
            if event.ty == "wasm" {
                for attr in event.attributes {
                    match attr.key.as_str() {
                        "_contract_address" => {
                            contracts
                                .entry(SLUG_DAO_DAO_CORE)
                                .or_insert_with(|| attr.value.clone());
                        }
                        "voting_module" => {
                            contracts
                                .entry(SLUG_DAO_VOTING_CW4)
                                .or_insert_with(|| attr.value.clone());
                        }
                        "prop_module" => {
                            contracts
                                .entry(SLUG_DAO_PROPOSAL_SINGLE_INSTANT)
                                .or_insert_with(|| attr.value.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        // TODO: Assert contracts keys are existing
        println!("contracts: {:?}", contracts);
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
                .join(format!("{}.wasm", filename)),
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
        let (app, contracts, admin, voters) = test_init(1);
        let wasm = Wasm::new(&app);

        // TODO: Mock signatures taking voter.publickey to recover the sig
        let mut vote_signatures: Vec<VoteSignature> = vec![];
        for voter in voters {
            println!("------------TT-------------");
            println!("Prefix: {:?}", voter.prefix());
            println!("Signing Key: {:?}", voter.signing_key().public_key());
            println!("Fee Setting: {:?}", voter.fee_setting());
            println!("Public_key : {:?}", voter.public_key());
            println!("Address : {:?}", voter.address());
            // let msg = "Hello World!";
            let mut msg = "17cd4a74d724d55355b6fb2b0759ca095298e3fd1856b87ca1cb2df5409058022736d21be071d820b16dfc441be97fbcea5df787edc886e759475469e2128b22f26b82ca993be6695ab190e673285d561d3b6d42fcc1edd6d12db12dcda0823e9d6079e7bc5ff54cd452dad308d52a15ce9c7edd6ef3dad6a27becd8e001e80f";

            let hash = compute_sha256_hash(msg);
            println!("hash: {:?}", hash);
            println!(
                "public key : {:?}, JSON - {:?}",
                voter.public_key(),
                voter.public_key().to_json().as_str()
            );
            println!(
                "public key BYTES : {:?} ",
                voter.public_key().to_bytes().as_slice()
            );
            println!(
                "public key ACCOUNT ID : {:?} ",
                voter.public_key().account_id("osmo")
            );
            println!("------------TT-------------");
            match hash {
                Ok(hash_str) => {
                    match hex::decode(&hash_str) {
                        Ok(hash_bytes) => {
                            // VoteSignature
                            vote_signatures.push(VoteSignature {
                                message_hash: hash_bytes,
                                signature: voter
                                    .signing_key()
                                    .sign(msg.as_bytes())
                                    .unwrap()
                                    .as_ref()
                                    .to_vec(),
                            })
                        }
                        Err(e) => {
                            // Handle the error, maybe with a panic or by returning an error
                            panic!("Error decoding hash: {}", e);
                        }
                    }
                }
                Err(e) => {
                    // Handle the error in hash computation, maybe with a panic or by returning an error
                    panic!("Error computing hash: {}", e);
                }
            }
        }

        // TODO: DEBUG, remove this
        for vote_signature in &vote_signatures {
            println!(
                "message_hash: {:?} \n signature {:?}",
                vote_signature.message_hash, vote_signature.signature
            );

            println!(
                "message_hash_str {:?}, \n signature_str {:?}",
                String::from_utf8_lossy(&vote_signature.message_hash).into_owned(),
                String::from_utf8_lossy(&vote_signature.signature).into_owned()
            )
        }
        println!("----------BEFORE CONTRACT EXECUTION--------");

        // TODO: Do Admin send from admin to treasury
        // TODO: Get Admin balance before

        // TODO: Do mock bank message from treasury to admin account

        // Execute execute_propose (proposal, voting and execution in one single workflow)
        let execute_propose_resp = wasm
            .execute(
                contracts.get(SLUG_DAO_PROPOSAL_SINGLE_INSTANT).unwrap(),
                &ExecuteMsg::Propose(SingleChoiceInstantProposeMsg {
                    title: "Title".to_string(),
                    description: "Description".to_string(),
                    msgs: vec![], // TODO: Mock a simple bank transfer that in prod will be the trigger exec to the middleware contract
                    proposer: None, // TODO: Some(admin.address()) is causing "pre-propose modules must specify a proposer. lacking one, no proposer should be specified: execute wasm contract failed"
                    vote_signatures,
                }),
                &vec![],
                &admin,
            )
            .unwrap();
        println!("execute_propose_resp: {:?}", execute_propose_resp);

        // TODO: Assert Admin balance after = (before + transfer_amount)

        // TODO: Assert proposal status after (closed, executed, deposit refunded, etc)
    }




    #[test]
    #[ignore]
    fn test_secp_sig_1() {
        println!("test: {:?}", "test_secp_sig_1");
        let (app, contracts, admin, voters) = test_init(1);
        let wasm = Wasm::new(&app);
        let mut vote_signatures: Vec<VoteSignature> = vec![];
        let mut vote_sig: VoteSignature;
         // let msg = "Hello World!";
        let mut msg = "17cd4a74d724d55355b6fb2b0759ca095298e3fd1856b87ca1cb2df5409058022736d21be071d820b16dfc441be97fbcea5df787edc886e759475469e2128b22f26b82ca993be6695ab190e673285d561d3b6d42fcc1edd6d12db12dcda0823e9d6079e7bc5ff54cd452dad308d52a15ce9c7edd6ef3dad6a27becd8e001e80f";
        let mut hash_result = compute_sha256_hash(msg);
        match &hash_result {
            Ok(hash) => {
                // Now 'hash' contains the computed hash string
                if *hash == "586052916fb6f746e1d417766cceffbe1baf95579bab67ad49addaaa6e798862" {
                    // Logic when hash matches
                    println!("Hash matches!");
                } else {
                    // Logic when hash does not match
                    println!("Hash does not match.");
                }
            }
            Err(e) => {
                // Handle error case
                eprintln!("Failed to compute hash: {}", e);
            }
        }

        println!("hash: {:?}", hash_result);


        let mut voter : &SigningAccount = &voters[0];
        // let mut pub_key :cosmrs::crypto::PublicKey  = voter.public_key();
        let mut pub_key  = voter.public_key();
        println!("------------voter info------------");

        println!("voter public key - {:?}", voter.public_key());
        println!("public key : {:?}, \n JSON - {:?}", voter.public_key(),voter.public_key().to_json().as_str() );
        println!("public key BYTES : {:?} ",  voter.public_key().to_bytes().as_slice() );
        println!("public key BYTES : {:?} ",  voter.public_key().account_id("osmo") );

        println!("public key String : {:?} ",  voter.public_key().to_string() );
        println!("public key ACCOUNT ID : {:?} ",  voter.public_key().account_id("osmo").unwrap().to_string() );
        let tm_key : cosmrs::tendermint::PublicKey = pub_key.try_into().expect("try_into failure");
        println!("tm_key {:?}", tm_key);
        let secp_key = tm_key.secp256k1().unwrap();
        println!("secp256k1 {:?}", secp_key);
        let bech_32_str = tm_key.to_bech32("osmo");
        println!("bech_32_str {:?}", bech_32_str);
        println!("to_hex - {:?}", tm_key.to_hex());
    }


}
