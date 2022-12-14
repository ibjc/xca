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

from web3 import Web3
from web3.middleware import construct_sign_and_send_raw_middleware
import requests

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
wallet = inj.wallet(InjKey(mnemonic=nuhmonik, coin_type=60))

################################################
# evm objects
################################################

rpc_url = "https://goerli.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161"
w3 = Web3(Web3.HTTPProvider(rpc_url))
private_key = "4030670b030e97abb0c3d71b6a8a3a07bb12c56bf0f0d2a8eab8e26d1f628626"
evm_wallet = w3.eth.account.privateKeyToAccount(private_key)

w3.middleware_onion.add(construct_sign_and_send_raw_middleware(evm_wallet))
w3.eth.default_account = evm_wallet.address

################################################
# deploy func
################################################

def deploy_local_wasm(file_path, wallet, inj):
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

def init_contract(code_id, init_msg, wallet, inj, name):

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

def execute_msg(address, msg, wallet, inj, coins=None):

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

def bank_msg_send(recipient, amount, wallet, inj):

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

def to_binary(msg):
  return base64.b64encode(json.dumps(msg).encode("utf-8")).decode("utf-8")

def proto_to_binary(msg):
  return base64.b64encode(msg.SerializeToString()).decode("utf-8")


def stargate_msg(type_url, msg, wallet, terra):
  account_data = inj.inj.query(f"/cosmos/auth/v1beta1/accounts/{wallet.key.acc_address}")

  account_number = int(account_data["account"]["base_account"]["account_number"])
  sequence = int(account_data["account"]["base_account"]["sequence"])
  chain_id = terra.chain_id

  # format msgs for tx
  tx_body = TxBody(
    messages=[
      Any(type_url=type_url, value=bytes(msg))
    ],
    memo="",
    timeout_height=0
  )

  # publish public key, create sign-document, and produce signature 
  signer_info = SignerInfo(
    public_key=wallet.key.public_key.pack_any(),
    mode_info=ModeInfo(
      single=ModeInfoSingle(
        mode=SignMode.SIGN_MODE_DIRECT
      )
    ),
    sequence=sequence,
  )

  auth_info = AuthInfo(
    signer_infos=[signer_info],
    fee=Fee(2000000,"500000uosmo").to_proto(),
  )

  sign_doc = SignDoc(
    body_bytes=bytes(tx_body),
    auth_info_bytes=bytes(auth_info),
    chain_id=chain_id,
    account_number=account_number
  )

  sk = SigningKey.from_string(wallet.key.private_key, curve=SECP256k1)
  signature = sk.sign_deterministic(
    data=bytes(sign_doc),
    hashfunc=hashlib.sha256,
    sigencode=sigencode_string_canonize,
  )

  # fabricate ready-to-send tx (messages, signer public info, signatures)
  tx = Tx(
    body=tx_body,
    auth_info=auth_info,
    signatures=[signature]
  )

  # post to lcd txs endpoint
  tx_result = terra.osmosis.broadcast(tx)

  return tx_result



################################################
# hardcoded contracts
################################################

inj_shouter_contract = "inj159fw63ffz8s9jspfr0ff3lkt3vanrjqxh59jl3"
inj_shouter_contract_standardized = "000000000000000000000000a152ed452911e05940291bd298fecb8b3b31c806"
inj_shouter_sequence = 0


evm_shouter_contract = "0x185589c355b1499a3775147d9f3c4f5e19154eee"
evm_shouter_contract_standardized = "000000000000000000000000" + evm_shouter_contract[2:]
evm_shouter_sequence = 0

abi_dict = {}
with open("/repos/xca/scripts/PuniswapV3Pool.json") as fp:
  abi_dict = json.load(fp)

evm_shouter_abi = abi_dict
evm_shouter_object = w3.eth.contract(abi=evm_shouter_abi["abi"], address=Web3.toChecksumAddress(evm_shouter_contract))

evm_wormhole_contract = "0x706abc4E45D419950511e474C7B9Ed348A4a716c"
evm_wormhole_contract_standardized = "0000000000000000000000003c3d457f1522d3540ab3325aa5f1864e34cba9d0"

abi_dict = {}
with open("/repos/xca/scripts/wormhole_core.json") as fp:
  abi_dict = json.load(fp)

evm_wormhole_abi = abi_dict
evm_wormhole_object = w3.eth.contract(abi=evm_wormhole_abi, address=Web3.toChecksumAddress(evm_wormhole_contract))

base_url = "https://wormhole-v2-testnet-api.certus.one/v1/signed_vaa/"

################################################
# main loop
################################################

attempts = 10

for _ in range(attempts):

    time.sleep(0.1)

    to_inj_vaa_resp = requests.get(f"{base_url}2/{evm_shouter_contract_standardized}/{evm_shouter_sequence}")

    if to_inj_vaa_resp.status_code == 200:
        #relay to injective

        print(f"relaying to inj: {to_inj_vaa_resp.text}")
        execute_msg(inj_shouter_contract, {"submit_vaa":{"vaa": to_inj_vaa_resp.json()["vaaBytes"]}}, wallet, inj)
        break
    evm_shouter_sequence += 1
    
for _ in range(attempts):
    time.sleep(0.1)
    
    to_evm_vaa_resp = requests.get(f"{base_url}19/{inj_shouter_contract_standardized}/{inj_shouter_sequence}")  
  
    if to_evm_vaa_resp.status_code == 200:
        #relay to goerli
        print(f"relaying to evm: {base64.b64decode(to_evm_vaa_resp.json()['vaaBytes'])}")
        print(f"relaying text to evm: {to_evm_vaa_resp.text}")
        evm_shouter_object.functions.receiveVAA(base64.b64decode(to_evm_vaa_resp.json()["vaaBytes"])).transact()
        break
    inj_shouter_sequence += 1

print("Complete!")
