################################################
# imports
################################################

import pandas as pd
import os
import yaml
from terra_sdk.client.lcd import LCDClient
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract, MsgMigrateContract
from terra_sdk.core.fee import Fee
from terra_sdk.core.bank.msgs import MsgSend
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.client.lcd.api.tx import CreateTxOptions
from terra_sdk.client.localterra import LocalTerra
from terra_sdk.core.coins import Coins, Coin
import base64
import json
import pendulum
import subprocess
import argparse
from terra_sdk.core.wasm.data import AccessConfig
from terra_proto.cosmwasm.wasm.v1 import AccessType
import time
from terra_sdk.core import AccAddress
from terra_sdk.core.bech32 import get_bech
from terra_sdk.client.lcd.api._base import BaseAsyncAPI, sync_bind

from terra_proto.cosmos.tx.v1beta1 import Tx, TxBody, AuthInfo, SignDoc, SignerInfo, ModeInfo, ModeInfoSingle, BroadcastTxResponse
from terra_proto.cosmos.tx.signing.v1beta1 import SignMode
from betterproto.lib.google.protobuf import Any

from ecdsa import SECP256k1, SigningKey
from ecdsa.util import sigencode_string_canonize
import hashlib

################################################
# terra objects
################################################

terra = LCDClient(url="https://pisco-lcd.terra.dev/", chain_id="pisco-1")

nuhmonik = " ".join(["zebra" for x in range(24)])
wallet = terra.wallet(MnemonicKey(mnemonic=nuhmonik))

nuhmonik1 = " ".join(["rifle" for x in range(24)])
wallet1 = terra.wallet(MnemonicKey(mnemonic=nuhmonik1))

nuhmonik2 = " ".join(["differ" for x in range(24)])
wallet2 = terra.wallet(MnemonicKey(mnemonic=nuhmonik2))

################################################
# inj objects
################################################

class AsyncInjAPI(BaseAsyncAPI):

    async def query(self, query_string: str):
        res = await self._c._get(query_string)
        return res

    async def broadcast(self, tx):
        res = await self._c._post("/cosmos/tx/v1beta1/txs", {"tx_bytes": proto_to_binary(tx), "mode": "BROADCAST_MODE_BLOCK"})
        return res


class InjAPI(AsyncInjAPI):

    @sync_bind(AsyncInjAPI.query)
    # see https://lcd-test.osmosis.zone/swagger/#/
    def query(self, query_string: str):
        pass

    @sync_bind(AsyncInjAPI.broadcast)
    def broadcast(self, tx: Tx):
        pass

inj = LCDClient(url="https://k8s.testnet.lcd.injective.network:443", chain_id="localterra")
inj.chain_id = "injective-888"
inj.inj = InjAPI(inj)

#override terra prefix
class InjKey(MnemonicKey):
  @property
  def acc_address(self) -> AccAddress: 
    if not self.raw_address:
      raise ValueError("could not compute acc_address: missing raw_address")
    return AccAddress(get_bech("inj", self.raw_address.hex()))


nuhmonik = "differ flight humble cry abandon inherit noodle blood sister potato there denial woman sword divide funny trash empty novel odor churn grid easy pelican"
inj_wallet = inj.wallet(InjKey(mnemonic=nuhmonik, coin_type=60))

nuhmonik1 = " ".join(["rifle" for x in range(24)])
inj_wallet1 = inj.wallet(InjKey(mnemonic=nuhmonik1, coin_type=60))

nuhmonik2 = " ".join(["differ" for x in range(24)])
inj_wallet2 = inj.wallet(InjKey(mnemonic=nuhmonik2, coin_type=60))


################################################
# deploy func
################################################

def deploy_local_wasm(file_path, wallet, terra):
  with open(file_path, "rb") as fp:
    file_bytes = base64.b64encode(fp.read()).decode()
    store_code_msg = MsgStoreCode(wallet.key.acc_address, file_bytes, instantiate_permission=AccessConfig(AccessType.ACCESS_TYPE_EVERYBODY, ""))
    store_code_tx = wallet.create_and_sign_tx(CreateTxOptions(msgs=[store_code_msg]))
    store_code_result = terra.tx.broadcast(store_code_tx)

  #persist code_id
  deployed_code_id = store_code_result.logs[0].events_by_type["store_code"]["code_id"][0]

  return deployed_code_id

def init_contract(code_id, init_msg, wallet, terra, name):

  #invoke contract instantiate
  instantiate_msg = MsgInstantiateContract(
    wallet.key.acc_address,
    wallet.key.acc_address,
    code_id,
    name,
    init_msg,
  )

  #there is a fixed UST fee component now, so it's easier to pay fee in UST
  instantiate_tx = wallet.create_and_sign_tx(CreateTxOptions(msgs=[instantiate_msg]))
  instantiate_tx_result = terra.tx.broadcast(instantiate_tx)

  return instantiate_tx_result

def execute_msg(address, msg, wallet, terra, coins=None):

  execute_msg = MsgExecuteContract(
    sender=wallet.key.acc_address,
    contract=address,
    msg=msg,
    coins=coins 
  )

  tx = wallet.create_and_sign_tx(CreateTxOptions(msgs=[execute_msg]))
  tx_result = terra.tx.broadcast(tx)

  return tx_result

def migrate_msg(contract_address, new_code_id, msg, wallet, terra):
  migrate_msg = MsgMigrateContract(
    sender=wallet.key.acc_address,
    contract=contract_address,
    code_id=new_code_id,
    msg=msg,
  )

  tx = wallet.create_and_sign_tx(CreateTxOptions(msgs=[migrate_msg]))
  tx_result = terra.tx.broadcast(tx)

  return tx_result

def bank_msg_send(recipient, amount, wallet, terra):

  bank_msg = MsgSend(
    from_address=wallet.key.acc_address,
    to_address=recipient,
    amount=amount,
  )

  #there is a fixed UST fee component now, so it's easier to pay fee in UST
  tx = wallet.create_and_sign_tx(CreateTxOptions(msgs=[bank_msg]))
  tx_result = terra.tx.broadcast(tx)

  return tx_result

def to_binary(msg):
  return base64.b64encode(json.dumps(msg).encode("utf-8")).decode("utf-8")

def proto_to_binary(msg):
  return base64.b64encode(msg.SerializeToString()).decode("utf-8")



################################################
# deploy inj func
################################################

def inj_deploy_local_wasm(file_path, wallet, inj):
  with open(file_path, "rb") as fp:
    file_bytes = base64.b64encode(fp.read()).decode()
    store_code_msg = MsgStoreCode(wallet.key.acc_address, file_bytes, instantiate_permission=AccessConfig(AccessType.ACCESS_TYPE_EVERYBODY, ""))


    account_data = inj.inj.query(f"/cosmos/auth/v1beta1/accounts/{wallet.key.acc_address}")

    opts = CreateTxOptions(msgs=[store_code_msg], fee=Fee(5000000, "3000000000000000inj"))
    opts.account_number = int(account_data["account"]["base_account"]["account_number"])
    opts.sequence = int(account_data["account"]["base_account"]["sequence"])
    store_code_tx = wallet.create_and_sign_tx(opts)
    store_code_result = inj.tx.broadcast(store_code_tx)

  #persist code_id
  #print(store_code_result)
  deployed_code_id = store_code_result.logs[0].events_by_type["store_code"]["code_id"][0]

  return deployed_code_id

def inj_init_contract(code_id, init_msg, wallet, inj, name):

  #invoke contract instantiate
  instantiate_msg = MsgInstantiateContract(
    wallet.key.acc_address,
    wallet.key.acc_address,
    code_id,
    name,
    init_msg,
  )

  account_data = inj.inj.query(f"/cosmos/auth/v1beta1/accounts/{wallet.key.acc_address}")

  opts = CreateTxOptions(msgs=[instantiate_msg], fee=Fee(3000000, "1500000000000000inj"))
  opts.account_number = int(account_data["account"]["base_account"]["account_number"])
  opts.sequence = int(account_data["account"]["base_account"]["sequence"])

  #there is a fixed UST fee component now, so it's easier to pay fee in UST
  instantiate_tx = wallet.create_and_sign_tx(opts)
  instantiate_tx_result = inj.tx.broadcast(instantiate_tx)

  return instantiate_tx_result

def inj_execute_msg(address, msg, wallet, inj, coins=None):

  execute_msg = MsgExecuteContract(
    sender=wallet.key.acc_address,
    contract=address,
    msg=msg,
    coins=coins 
  )

  account_data = inj.inj.query(f"/cosmos/auth/v1beta1/accounts/{wallet.key.acc_address}")

  opts = CreateTxOptions(msgs=[execute_msg], fee=Fee(3000000, "1500000000000000inj"))
  opts.account_number = int(account_data["account"]["base_account"]["account_number"])
  opts.sequence = int(account_data["account"]["base_account"]["sequence"])

  tx = wallet.create_and_sign_tx(opts)
  tx_result = inj.tx.broadcast(tx)

  return tx_result

def inj_bank_msg_send(recipient, amount, wallet, inj):

  bank_msg = MsgSend(
    from_address=wallet.key.acc_address,
    to_address=recipient,
    amount=amount,
  )

  account_data = inj.inj.query(f"/cosmos/auth/v1beta1/accounts/{wallet.key.acc_address}")

  opts = CreateTxOptions(msgs=[m], fee=Fee(3000000, "1500000000000000inj"))
  opts.account_number = int(account_data["account"]["base_account"]["account_number"])
  opts.sequence = int(account_data["account"]["base_account"]["sequence"])

  #there is a fixed UST fee component now, so it's easier to pay fee in UST
  tx = wallet.create_and_sign_tx(opts)
  tx_result = inj.tx.broadcast(tx)

  return tx_result


################################################
# deploy code id
################################################

#terra-side
terra_registry_id = deploy_local_wasm("/repos/xca/artifacts/registry.wasm", wallet, terra)
terra_account_id = deploy_local_wasm("/repos/xca/artifacts/account.wasm", wallet, terra)

terra_multisig_code_id = deploy_local_wasm("/repos/cw-plus/artifacts/cw3_fixed_multisig.wasm", wallet, terra)

terra_staking_id = deploy_local_wasm("/repos/xca/artifacts/staking.wasm", wallet, terra)

#inj-side
inj_registry_id = inj_deploy_local_wasm("/repos/xca/artifacts/registry.wasm", inj_wallet, inj)
inj_account_id = inj_deploy_local_wasm("/repos/xca/artifacts/account.wasm", inj_wallet, inj)

inj_multisig_code_id = inj_deploy_local_wasm("/repos/cw-plus/artifacts/cw3_fixed_multisig.wasm", inj_wallet, inj)

inj_staking_id = inj_deploy_local_wasm("/repos/xca/artifacts/staking.wasm", inj_wallet, inj)

################################################
# configs
################################################

terra_wormhole_core_contract = "terra19nv3xr5lrmmr7egvrk2kqgw4kcn43xrtd5g0mpgwwvhetusk4k7s66jyv0"
terra_xca_factory = "terra19nv3xr5lrmmr7egvrk2kqgw4kcn43xrtd5g0mpgwwvhetusk4k7s66jyv0"

inj_wormhole_core_contract = "inj1xx3aupmgv3ce537c0yce8zzd3sz567syuyedpg"
inj_xca_factory = "inj1xx3aupmgv3ce537c0yce8zzd3sz567syuyedpg"

################################################
# registry init
################################################


#create registry on terra
init_registry_terra = {
  "chain_id_here": 18,
  "x_account_code_id": int(terra_account_id)
}

init_result = init_contract(terra_registry_id, init_registry_terra, wallet, terra, "terra_registry")
terra_registry_address= init_result.logs[0].events_by_type["instantiate"]["_contract_address"][0]

execute_msg(terra_registry_address, {"upsert_chain_info": {
  "chain_info":{
  "wormhole_id": 18,
  "wormhole_core": terra_wormhole_core_contract,
  "x_account_factory": wallet.key.acc_address,
  "x_account_deployer": wallet.key.acc_address,
}}}, wallet, terra)

execute_msg(terra_registry_address, {"upsert_chain_info": {
  "chain_info":{
  "wormhole_id": 19,
  "wormhole_core": inj_wormhole_core_contract,
  "x_account_factory": inj_wallet.key.acc_address,
  "x_account_deployer": inj_wallet.key.acc_address,
}}}, wallet, terra)


#create registry on inj
init_registry_inj = {
  "chain_id_here": 19,
  "x_account_code_id": int(inj_account_id)
}

init_result = inj_init_contract(inj_registry_id, init_registry_inj, inj_wallet, inj, "inj_registry")
inj_registry_address= init_result.logs[0].events_by_type["instantiate"]["_contract_address"][0]

inj_execute_msg(inj_registry_address, {"upsert_chain_info": {
  "chain_info":{
  "wormhole_id": 18,
  "wormhole_core": terra_wormhole_core_contract,
  "x_account_factory": wallet.key.acc_address,
  "x_account_deployer": wallet.key.acc_address,
}}}, inj_wallet, inj)

inj_execute_msg(inj_registry_address, {"upsert_chain_info": {
  "chain_info":{
  "wormhole_id": 19,
  "wormhole_core": inj_wormhole_core_contract,
  "x_account_factory": inj_wallet.key.acc_address,
  "x_account_deployer": inj_wallet.key.acc_address,
}}}, inj_wallet, inj)

################################################
# account deploy
################################################

#create xaccount on terra
init_xaccount_terra = {
  "x_chain_registry_address": terra_registry_address,
  "admin": {
    "chain_id": 19,
    "address": inj_wallet.key.acc_address,
  },
  "master": {
    "chain_id": 19,
    "address": inj_wallet.key.acc_address,
  },

}

init_result = init_contract(terra_account_id, init_xaccount_terra, wallet, terra, "terra_xaccount_deployer")
terra_xaccount_address= init_result.logs[0].events_by_type["instantiate"]["_contract_address"][0]

#create registry on inj
init_xaccount_inj = {
  "x_chain_registry_address": inj_registry_address,
  "admin": {
    "chain_id": 19,
    "address": inj_wallet.key.acc_address,
  },
  "master": {
    "chain_id": 19,
    "address": inj_wallet.key.acc_address,
  },

}

init_result = inj_init_contract(inj_account_id, init_xaccount_inj, inj_wallet, inj, "inj_xaccount_deployer")
inj_xaccount_address= init_result.logs[0].events_by_type["instantiate"]["_contract_address"][0]


terra.wasm.contract_query(terra_wormhole_core_contract, {"query_address_hex":{"address": terra_xaccount_address}})

#init contract on inj
message = {
  "call":{
    "outgoing_envelope":{
      "destination_chain": 18,
      "destination_address": 
      "4fae631a8fe88c178290fd7ae66abd8f59c96992d3a824315dcb68ce6c2c27f3",
      "is_response_expected": 0,
      "is_executable": 1,
      "response_of":{
        "chain_id": 18,
        "sequence": 2,
      },
    },
    "msg_type": "ExecuteMsg",
    "msg": to_binary({
            "denom_name": "uluna",
          }),
      }
  }
inj_execute_msg(inj_xaccount_address, message, inj_wallet, inj)



inj_xaccount_address_standardized = "00000000000000000000000098582c96514cdfb7080db7d2982eaf6591d48f21"
inj_xaccount_sequence = 0
base_url = "https://wormhole-v2-testnet-api.certus.one/v1/signed_vaa/"

to_terra_vaa_resp = requests.get(f"{base_url}19/{inj_xaccount_address_standardized}/{inj_xaccount_sequence}")


result = execute_msg(terra_xaccount_address, {"finish_call":{"vaas":[to_terra_vaa_resp.json()["vaaBytes"]]}}, wallet, terra)

################################################
# cw3 deploy
################################################


init_msg = {
  "max_voting_period": {"height": 100},
  "threshold": {
    "absolute_count" : {"weight": 2},
  },
  "voters":[
    {"addr": wallet1.key.acc_address, "weight": 1},
    {"addr": wallet2.key.acc_address, "weight": 1},
    {"addr": wallet.key.acc_address, "weight": 1},
  ]
}

cw3_result = init_contract(terra_multisig_code_id, init_msg, wallet, terra, "terra_cw3")
terra_cw3_address = cw3_result.logs[0].events_by_type["instantiate"]["_contract_address"][0]


init_msg = {
  "max_voting_period": {"height": 100},
  "threshold": {
    "absolute_count" : {"weight": 2},
  },
  "voters":[
    {"addr": inj_wallet1.key.acc_address, "weight": 1},
    {"addr": inj_wallet2.key.acc_address, "weight": 1},
    {"addr": inj_wallet.key.acc_address, "weight": 1},
  ]
}

cw3_result = inj_init_contract(inj_multisig_code_id, init_msg, inj_wallet, inj, "inj_cw3")
inj_cw3_address = cw3_result.logs[0].events_by_type["instantiate"]["_contract_address"][0]


"""
################################################
# make, vote, execute proposal
################################################

message = {
  "propose":{
    "title": "test",
    "description": "test69",
    "msgs":[
      {"bank": {"send":{"to_address": worker_wallet.key.acc_address, "amount":[{"denom":"uusd", "amount":"69000000"}]}}}
    ]
  }
}

result = execute_msg(terra_cw3_address, message, wallet, terra)
proposal_id = int(result.logs[0].events_by_type["wasm"]["proposal_id"][0])

vote_result = execute_msg(terra_cw3_address, {"vote":{"proposal_id":proposal_id, "vote": "yes"}}, wallet1, terra)

execute_result = execute_msg(terra_cw3_address, {"execute":{"proposal_id":proposal_id}}, wallet2, terra)
"""

################################################
# staking init
################################################


#create registry on terra
init_staking_terra = {
  "denom_name": "uluna",
}

init_result = init_contract(terra_staking_id, init_staking_terra, wallet, terra, "terra_staking")
terra_staking_address= init_result.logs[0].events_by_type["instantiate"]["_contract_address"][0]

coinz = Coins.from_str("1000000uluna");
execute_msg(terra_staking_address, {"stake":{}}, wallet, terra, coinz)


#create registry on inj
init_staking_inj = {
  "denom_name": "inj",
}

init_result = inj_init_contract(inj_staking_id, init_staking_inj, inj_wallet, inj, "inj_staking")
inj_staking_address = init_result.logs[0].events_by_type["instantiate"]["_contract_address"][0]

coinz = Coins.from_str("69inj");
inj_execute_msg(inj_staking_address, {"stake":{}}, inj_wallet, inj, coinz)