#!/bin/bash

# Set Osmosis Testnet variables
export CHAIN_ID="osmo-test-5"
export TESTNET_NAME="osmosis-testnet"
export DENOM="uosmo"
export BECH32_HRP="osmo"
export CONFIG_DIR=".osmosis"
export BINARY="osmosisd"
export RPC="https://rpc.testnet.osmosis.zone:443"

# Set the contract specific variables

export CODE_ID_CW_CORE=4897
export CW_CORE_LABEL="qsr_rebalance_dao"

export CODE_ID_PROP_SINGLE=4902  # Example code ID for Proposal Module
export CODE_ID_PROP_SINGLE_INSTANT=5761  # Example code ID for Proposal Module
export LABEL_PROP_SINGLE="qsr_dao_proposal_single"
export LABEL_PROP_SINGLE_INSTANT="qsr_dao_proposal_single_instant"
export CODE_ID_VOTING=4903  # Example code ID for Voting Module
export LABEL_VOTING="qsr_dao_voting"
export ADMIN_FACTORY_CONTRACT_ADDR="osmo1v5k3527dt2vt67848h8jk0az9dyl8sunsqaapznf2j9tm4arxxfs7gwa0n"  # Contract address variable
export ADMIN=null  # Admin address
export TXFLAG_1="--chain-id ${CHAIN_ID} --gas-prices 0.025uosmo --gas auto --gas-adjustment 1.3 --broadcast-mode block"
export TXFLAG_2="--chain-id ${CHAIN_ID} --gas 25000000 --fees 300000uosmo --broadcast-mode block --no-admin"
export NODE="https://rpc.testnet.osmosis.zone:443"


###############################################################
# # DAO-PROPOSAL-SINGLE MESSAGE PREP
###############################################################
PROP_MODULE_MSG_1='{
        "allow_revoting": false,
        "max_voting_period": {
          "time": 3600
        },
        "close_proposal_on_execution_failure": true,
        "pre_propose_info": {"AnyoneMayPropose":{}},
        "only_members_execute": true,
        "threshold": {
          "threshold_quorum": {
            "quorum": {
              "percent": "0.20"
            },
            "threshold": {
              "majority": {}
            }
          }
        }
      }'

echo $PROP_MODULE_MSG_1 | jq > "prop_message_1.json"
ENCODED_PROP_MESSAGE_1=$(echo -n $PROP_MODULE_MSG_1 | tr -d '[:space:]' | openssl base64 | tr -d '[:space:]')
echo $ENCODED_PROP_MESSAGE_1 > "encoded_prop_message_1.txt"
echo -e "\nENCODED PROP MESSAGE 1:\n$ENCODED_PROP_MESSAGE_1"


###############################################################
# # DAO-PROPOSAL-SINGLE-INSTANT MESSAGE PREP
###############################################################
PROP_MODULE_MSG_2='{
        "allow_revoting": false,
        "max_voting_period": {
          "height": 1
        },
        "close_proposal_on_execution_failure": true,
        "pre_propose_info": {
          "AnyoneMayPropose":{}
        },
        "only_members_execute": true,
        "threshold": {
          "absolute_count": {
              "threshold": "2"
          }
        }
      }'

echo $PROP_MODULE_MSG_2 | jq > "prop_message_2.json"
ENCODED_PROP_MESSAGE_2=$(echo -n $PROP_MODULE_MSG_2 | tr -d '[:space:]' | openssl base64 | tr -d '[:space:]')
echo $ENCODED_PROP_MESSAGE_2 > "encoded_prop_message_2.txt"
echo -e "\nENCODED PROP MESSAGE 2:\n$ENCODED_PROP_MESSAGE_2"


###############################################################
# VOTING MODULE MESSAGE PREPARATION
###############################################################
# TODO - Need to change the addresses here as per our use case for testings.
VOTING_MSG='{
  "group_contract": {
    "new": {
      "cw4_group_code_id": '"${CODE_ID_VOTING}"',
      "initial_members": [
        {
          "addr": "osmo1pqw0e5kykpzat033wu76pzg6sxhchfj36ytpf4",
          "weight": 0
        },
        {
          "addr": "osmo1p577t98ggzlucph3wwehmtmhqr75fda0rzuwwq",
          "weight": 1
        },
        {
          "addr": "osmo17qe4cuv83dvz99w967pjtu5fzusy8mqrklmuw7",
          "weight": 1
        },
        {
          "addr": "osmo1wzdhlvurmav577eu7n3z329eg5ykaez0jh50ug",
          "weight": 1
        },
        {
          "addr": "osmo16a8tgcgx4362f5x4lsq4k2gtz9p9fjn7rgzl0l",
          "weight": 1
        },
        {
          "addr": "osmo1zntq6hau8y555qv3razpu9xnrpyjdtt94ufahy",
          "weight": 1
        }
      ]
    }
  }
}'

echo $VOTING_MSG | jq > "voting_msg.json"
ENCODED_VOTING_MESSAGE=$(echo $VOTING_MSG | tr -d '[:space:]' | openssl base64 | tr -d '[:space:]')
echo $ENCODED_VOTING_MESSAGE > "encoded_voting_message.txt"
echo -e "\nENCODED VOTING MESSAGE:\n$ENCODED_VOTING_MESSAGE"

###############################################################
# CW CORE INIT MESSAGE PREPARATION
###############################################################
CW_CORE_INIT='{
  "admin": '${ADMIN}',
  "automatically_add_cw20s": true,
  "automatically_add_cw721s": true,
  "description": "QSR Rebalance DAO",
  "name": "QSR Rebalance DAO",
  "proposal_modules_instantiate_info": [
    {
      "admin": {
        "core_module": {}
      },
      "code_id": '${CODE_ID_PROP_SINGLE}',
      "label": "'${LABEL_PROP_SINGLE}'",
      "msg": "'${ENCODED_PROP_MESSAGE_1}'",
       "funds": []
    },
    {
      "admin": {
          "core_module": {}
      },
      "code_id": '${CODE_ID_PROP_SINGLE_INSTANT}',
      "label": "'${LABEL_PROP_SINGLE_INSTANT}'",
      "msg": "'${ENCODED_PROP_MESSAGE_2}'",
      "funds": []
      }
  ],
  "voting_module_instantiate_info": {
    "admin": {
      "core_module": {}
    },
    "code_id": '${CODE_ID_VOTING}',
    "label": "'${LABEL_VOTING}'",
    "msg": "'${ENCODED_VOTING_MESSAGE}'",
     "funds": []
  }
}'
echo "CW_CORE_INIT = $CW_CORE_INIT"
echo $CW_CORE_INIT | jq > "cw_core_init.json"
CW_CORE_STRIPPED=$(echo -n $CW_CORE_INIT | tr -d '[:space:]')
echo -e "CW-CORE INSTANTIATE MESSAGE:\n$CW_CORE_STRIPPED"
CW_CORE_ENCODED=$(echo -n $CW_CORE_STRIPPED | openssl base64 | tr -d '[:space:]')
echo $CW_CORE_ENCODED > "cw_core_encoded.txt"
echo -e "\nCW-CORE ENCODED MESSAGE:\n$CW_CORE_ENCODED"

# init with factory
INIT_MSG="{\"instantiate_contract_with_self_admin\":{\"code_id\":${CODE_ID_CW_CORE}, \"label\": \"${CW_CORE_LABEL}\", \"instantiate_msg\":\"$CW_CORE_ENCODED\"}}"
echo $INIT_MSG | jq > "cw_core_init_with_factory.json"
echo -e "INIT MESSAGE:\n$INIT_MSG"

#####################################################
# Not running the command from the script fo now due to keying not known for the user.

# instantiate with factory
# echo 'instantiating cw-core with factory'
# NOTE - Commented the command part to do it manually
# ${BINARY} tx wasm execute $ADMIN_FACTORY_CONTRACT_ADDR "$INIT_MSG" --from $KEY_NAME --node $NODE $TXFLAG_2

# echo 'instantiating cw-core without factory'
# NOTE - Commented the command part to do it manually
# ${BINARY} tx wasm instantiate $CODE_ID_CW_CORE "$CW_CORE_INIT" --from $KEY_NAME --node $NODE $TXFLAG_2

