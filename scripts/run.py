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

################################################
# terra objects
################################################

terra_local = LocalTerra()
terra = LCDClient(url="https://pisco-lcd.terra.dev/", chain_id="pisco-1")

nuhmonik = " ".join(["zebra" for x in range(24)])
testnet_deployer_wallet = terra.wallet(MnemonicKey(mnemonic=nuhmonik))

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

def bank_msg_send(recipient, amount, wallet, terrra):

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
# deploy code id
################################################

wormhole_code_id = deploy_local_wasm("/repos/xca/artifacts/xca.wasm", testnet_deployer_wallet, terra)

