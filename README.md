# Cinnabar Blind-Box Demo

> note: built from [ckb-scrpt-templates](https://github.com/cryptape/ckb-script-templates), which is the current suggested CKB contract template, but only serves for writting contracts.

The blind-box, powered by Cinnabar, has two functions:
- allow users to create blind-box purchase cells under standalone server's authority, which represents the request of purhcase, known as `Purchase` intention.
- allow standalone server to consume purchase cells and create asset cells for users, for example, open blind-box to get Spore DOBs.

Refer to the calculate and verify philosophy of Cinnabar, project blind-box consists of the [contract](contracts/blind-box-type/src/main.rs) implementation, which known as `verify`, and the [calculator](calculator/src/lib.rs) interface, which known as `calculate`.

The calculator interfaces cover up all details the blind-box contract required from users, they help assemble a minimal particular transaction sturcture via providing Cinnabar instruction, which is composable.

The verifier, which is exactly the contract implementation of blind-box, is done in a form of verfication tree and follows below workflow:
- Collect parameters as global context (entry)
  - Check validity of purhcase intention if blind-box script only shows up in Outputs (purchase)
  - Check validity of open intention if blind-box script only shows up in Inputs (open)

> note: calculator crate is exported from this monorepo, so if this is a public Github project, anyone can refer it to create blind-box purchase transaction easily and directly.

## Compile and Deploy

Compile blind-box contract:
```bash
$ make build
```

And then, compiled RISC-V binary will be located in `./build/release` directory.

Deploy blind-box contract on testnet (ckb-cli required):
```bash
$ cargo deploy --contract-name blind-box-type --payer-address <your-address-from-ckb-cli> --tag v0.2.0 --type-id

# deploy blind-box-type contract on testnet (default) with type-id feature opened
```

And then, the deployment transaction cache file will generate in `./deployment/txs/` directory, as we use testnet to deploy on, the final deployment information file will be in `./deployment/testnet/` directory (as `./deployment/mainnet/` if mainnet figgered).

The deployment information file contains a history list of each operations, which are deploy, migrate and consume, unlike the deployment feature of ckb-cli, every deployment file is independently generated, which isn't version management friendly.

Here's a part of pre-generated [deployment](deployment/testnet/blind-box-type.json) file on testnet:
```json
[
  {
    "name": "blind-box-type",
    "date": "2024-08-15T08:49:58.860943+00:00",
    "operation": "deploy",
    "version": "v0.1.1",
    "tx_hash": "0xf1731b7c020c1aac4e9c8d74e245d362c551488bc4f4ef4e7f0ccdcaa0437d50",
    "out_index": 0,
    "data_hash": "0x4644de948f08b1e2d76ab3d421ebb352c1ac55b822ccd813b2e2343eec906116",
    "occupied_capacity": 4697300000000,
    "payer_address": "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq28phxutezqvjgfv5q38gn5kwek4m9km3cmajeqs",
    "contract_owner_address": "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq28phxutezqvjgfv5q38gn5kwek4m9km3cmajeqs",
    "type_id": null,
    "__comment": "deploy the initial blind-box-type contract (this item can change anytime without impact on serde)"
  },
  {
    "name": "blind-box-type",
    "date": "2024-08-15T08:53:52.213768+00:00",
    "operation": "migrate",
    "version": "v0.1.2",
    "tx_hash": "0x2e76f0100faf7325587cd76445acc3161af70af1dbaeebcbc7565dabd64fe578",
    "out_index": 0,
    "data_hash": "0x4644de948f08b1e2d76ab3d421ebb352c1ac55b822ccd813b2e2343eec906116",
    "occupied_capacity": 4703800000000,
    "payer_address": "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq28phxutezqvjgfv5q38gn5kwek4m9km3cmajeqs",
    "contract_owner_address": "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq28phxutezqvjgfv5q38gn5kwek4m9km3cmajeqs",
    "type_id": "0x9b4dc400b6c9cbbcef0fc1cf8478b5292116bed1d16a52649b0b83fc4e3b3f3d",
    "__comment": null
  },
  ...
]
```

> note: Since ckb-cinnabar-scripts-template already integrated Cinnabar deployment module into its executable part (refer to [main.rs](src/main.rs)), unlike Capsule, there's no confilicts if different versions of Cinnabar must be used in one machine.

## Contract Test

Testing contract needs to build a minimum operational transaction structure and run in a native simulated enironment, usually this isn't an easy thing for most newcomers, because of the overwhelming workload of fake transaction assembly.

And this is also weird for newcomers that they have to write two versions of transaction assembly but nearly the same as each other. One is to assemble transaction in real world, for instance, sending transaction to testnet or mainnet, the other is to assemble minimum operational fake transaction in simulation.

Why are there two similar versions? Because we use always-success lock to replace the traditional secp256k1-sighash-all lock, and out points of transaction Inputs and CellDeps are meaningless which can exactly set to random, besides, transaction in simulation isn't necessary to be balanced.

Since the `Calculate` philosophy of Cinnabar is to provide minimum operating conditions of running a transaction, it's intuitive to integrate a fake version into instruction's operations, like below examples of `AddBlindBoxPurchaseInputCell` and `AddBlindBoxCelldep` operations:

```rust
struct AddBlindBoxPurchaseInputCell {
    // some fields
}

#[async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxPurchaseInputCell {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> Result<()> {
        if rpc.network() == Network::Fake {
            // just create a fake purchase cell of always-success for test 
            // ...
            Ok(())
        } else {
            // search and add a purcase cell in real world
            // ...
            Ok(())
        }
    }
}

struct AddBlindBoxCelldep {
    // some fields
}

#[async_trait]
impl<T: RPC> Operation<T> for AddBlindBoxCelldep {
    async fn run(self: Box<Self>, rpc: &T, skeleton: &mut TransactionSkeleton) -> Result<()> {
        if rpc.network() == Network::Fake {
            // load contract binary from ./build/release directory
            // ...
            Ok(())
        } else {
            // read deployed contract transaction from deployment information file
            // ...
            Ok(())
        }
    }
}
```

After then, writing contract test cases is quite simple, and most importantly, the calculator keeps the maximum code reuse rate, which can not only be used in real world, but also in simulation.

Run pre-written [test cases](tests/src/tests.rs):
```bash
$ make test

# make sure `make build` run before
```

## Examples

Calculator crate has two examples, which are for blind-box purchase and open demostration.

Run purchase example:
```bash
$ cd calculator
$ cargo run --example purchase <buyer-address> <server-address>
```

Run open example:
```bash
$ cd calculator
$ cargo run --example open <server-address>
```

> note: both purhcase and open examples run on CKB testnet and require ckb-cli installed
