## xAccount contract

In wormhole there are xAssets, xApps, xData. This introduces a new primitive to Wormhole to extend new types of functionality.
Sits on source and destination chains connected by Wormhole. They send and receive messages from each other.

Message examples:

- Receive a message from multisig contract on source chain to send to staking destination chain.
- Receive a message from staking contract on destination chain to submit governance proposal.
