<br>

<p align="center">
  <img src="./assets/medieval-100.png" width="100" alt="0xzero.org" />
</p>
<br>

<p align="center">
   <a href="https://github.com/0xZeroLabs/the-forge/network/members"><img src="https://img.shields.io/github/forks/0xZeroLabs/the-forge?style=for-the-badge&color=a8c7ff&labelColor=1a1b1f"></a>
   <img src="https://img.shields.io/github/stars/0xZeroLabs/the-forge?style=for-the-badge&logo=github&color=e6c419&labelColor=1d1b16">
   <a href="https://x.com/0xZeroOrg"><img src="https://img.shields.io/twitter/follow/0xZeroLabs.svg?style=for-the-badge&logo=x&color=e6c419&labelColor=1d1b16"></a>
   <br>
   <img src="https://img.shields.io/github/languages/count/0xZeroLabs/the-forge?style=for-the-badge&color=ffb4a2&labelColor=201a19">
   <a href="https://github.com/0xZeroLabs/the-forge/issues"><img src="https://img.shields.io/github/issues/0xZeroLabs/the-forge?style=for-the-badge&color=ffb4a2&labelColor=201a19"></a>
   <a href="https://github.com/0xZeroLabs/the-forge/pulls"><img src="https://img.shields.io/github/issues-pr-raw/0xZeroLabs/the-forge?style=for-the-badge&color=ffb4a2&labelColor=201a19"></a>
   <a href="https://github.com/0xZeroLabs/the-forge/graphs/contributors"><img src="https://img.shields.io/github/contributors-anon/0xZeroLabs/the-forge?style=for-the-badge&color=ffb4a2&labelColor=201a19"></a>
   <img src="https://img.shields.io/github/languages/code-size/0xZeroLabs/the-forge?style=for-the-badge&color=ffb4a2&labelColor=201a19">
<br>
  <a href="https://docs.0xzero.org"><img src="https://img.shields.io/badge/docs-%F0%9F%93%84-blue?style=for-the-badge&color=ffb4a2&labelColor=201a19"></a>
  <a href="https://github.com/0xZeroLabs/the-forge/blob/master/LICENSE"><img src="https://img.shields.io/github/license/0xZeroLabs/the-forge?style=for-the-badge&color=ffb4a2&labelColor=201a19"></a>
</p>

# The Forge ⚒

The Forge is an AVS built on [EigenLayer](https://eigenlayer.xyz) through the [Othentic](https://othentic.xyz) stack, designed to verifiably register any IP on Story Protocol unleashing access to unlimited IP.

# 🎯 Grand Plan

We're currently focused on achieving the following to get a working version of The Forge:

- [ ] Prover with Jolt (Transcript verifier)
- [x] Story Protocol IP smart contracts
- [ ] IP registration in execution registrar via Alloy
- [ ] Attestation service ZKP verification ...

# ⚙️ Set Up

To set up the environment, create a `.env` file with the usual Othentic
configurations (see the `.env.example`).

# ✈️ Install the Othentic CLI

Installing Othentic CLI with `npm`:

```console
npm i -g @othentic/othentic-cli
```

Verify installation by the command:

```console
othentic-cli -h
```

# 🧰 Prerequisites

You need to register 3 self-deploy Operators with a minimum of 0.01 stETH.

- Deployer account:
  - A minimum of 1.5 holETH (Faucet)
  - A minimum of 5 Amoy MATIC (Faucet)
- Operator account x 3 (Script):
  - A minimum of 0.02 holETH on Holesky
- ERC-20 token address

# 📑 Contracts Deployment

To deploy the AVS’s on-chain components, run the following command:

```console
othentic-cli network deploy \\
    --erc20 0x73967c6a0904aA032C103b4104747E88c566B1A2 \\
    --l1-initial-deposit 1000000000000000000 \\
    --l2-initial-deposit 2000000000000000000 \\
    --name test-avs-name
```

# 🏋️‍♂️ Operators Setup

Register as an operator for both EigenLayer and the AVS

```console
othentic-cli operator register
```

# 🔁 Convert ETH into stETH [Optional]

This command converts 0.012 ETH into stETH before depositing it into EigenLayer pool:

```console
othentic-cli operator deposit --strategy stETH --shares 0.01 --convert 0.012
```

Activate your Operator by depositing into EigenLayer
Deposit 0.01 stETH into EigenLayer pool.

```console
othentic-cli operator deposit --strategy stETH --shares 0.01
```

✅ Your internal Operators are now ready to opt-in to your AVS.

## ▶️ Run the demo

We provide a sample docker-compose configuration which sets up the following
services:

- Aggregator node
- 3 Attester nodes
- Validation Service endpoint
- Execution Service endpoint
- Sync Shares of operators across layers

run:

```console
docker-compose up --build
```

> [!NOTE]
> This might take a few minutes when building the images

# 🚀 Executing a task

To execute a task we send a
POST request to the Task Performer service:

```console
curl -X POST <http://localhost:4003/task/execute>
```

✅ Your demo AVS is functional!

### Updating the Othentic node version

To update the `othentic-cli` inside the docker images to the latest version, you
need to rebuild the images using the following command:

```console
docker-compose build --no-cache
```

## 🏗️ Architecture

The Othentic Attester nodes communicate with an AVS WebAPI endpoint which
validates tasks on behalf of the nodes. The attesters then sign the tasks based
on the AVS WebAPI response.

Attester nodes can either all communicate with a centralized endpoint or each
implement their own validation logic.

### AVS WebAPI

```
POST task/validate returns (bool) {"proofOfTask": "{proofOfTask}"};
```

### Sync Shares

sync the shares of operators between L1 and L2 at a fixed interval. The default interval is 12h and can be modified inside the docker-compose file
