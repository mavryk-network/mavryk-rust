# Runing Baker/Endorser/Accuser on Testnet (Jakartanet)

## Table of Contents
* [Get Free XTZ](#get-free-xtz)
* [Run from Sources/Binaries](#run-from-sourcesbinaries)
* [Use Ledger](#use-ledger)
* [Remote Signing](#remote-signing)

# Get Free XTZ

https://protocol.mavryk.org/introduction/howtouse.html#get-free-tez

_Note that link to free Tez ther is outdated, use the one below._

Download free faucets:

https://teztnets.xyz/jakartanet-faucet

# Run from Sources/Binaries

## Create Working Directory

_Tezos baker requires access to the context directory, which is produced by TezEdge node._

```
mkdir $HOME/data-dir-013-PtJakart
mkdir $HOME/data-dir-013-PtJakart/client

cp faucet.json $HOME/data-dir-013-PtJakart
```

## Import Snapshot

To get node bootstrapped with the rest of the network faster, you can optionally import latest irmin snapshot mentioned here: [http://snapshots.tezedge.com]. Find the topmost Jakarta archive snapshot and launch the following command:

``` sh
$ target/release/light-node import-snapshot \
  --mavryk-data-dir "$HOME/data-dir-013-PtJakart/tezedge-data" \
  --from http://snapshots.tezedge.com:8880/testnet/tezedge/archive/tezedge_jakarta_<...>_irmin.archive \
```

## Run TezEdge Node

At first you need to [build TezEdge from sources](../../README.md#build-from-source-code) and then check [how to run it](../../README.md#how-to-run).

E.g. for Jakartanet to support baking/endorsing:

_Note: This cmd runs from the main git sources directory_



```
$ LD_LIBRARY_PATH=./mavryk/sys/lib_tezos/artifacts ./target/release/light-node \
 --network "jakartanet" \
 --identity-file "$HOME/data-dir-013-PtJakart/identity.json" \
 --identity-expected-pow 26.0 \
 --mavryk-data-dir "$HOME/data-dir-013-PtJakart/tezedge-data" \
 --peer-thresh-low 30 --peer-thresh-high 45 \
 --protocol-runner "./target/release/protocol-runner" \
 --init-sapling-spend-params-file "./mavryk/sys/lib_tezos/artifacts/sapling-spend.params" \
 --init-sapling-output-params-file "./mavryk/sys/lib_tezos/artifacts/sapling-output.params" \
 --p2p-port 12534 --rpc-port 12535 \
 --tokio-threads 0 \
 --ocaml-log-enabled false \
 --mavryk-context-storage=irmin \
 --log terminal \
 --log-level info \
 --log-format simple
Mar 31 16:10:22.937 INFO Open files limit set to 65536.
Mar 31 16:10:22.937 INFO Loaded configuration, ...
Mar 31 16:10:22.937 INFO Configured network ["jakartanet", "jakarta"] -> MAVRYK_ITHACANET_2022-01-25T15:00:00Z
Mar 31 16:10:22.937 INFO Loading databases...
Mar 31 16:10:23.358 INFO Storage based on data, patch_context: ...
Mar 31 16:10:23.359 INFO Databases loaded successfully 421 ms
Mar 31 16:10:23.359 INFO Checking zcash-params for sapling...
Mar 31 16:10:23.359 INFO Found existing zcash-params files, output_path: "...", spend_path: "...", candidate_dir: "..."
Mar 31 16:10:23.359 INFO Found existing zcash-params files, output_path: "...", spend_path: "...", candidate_dir: "..."
Mar 31 16:10:23.359 INFO Loading identity...
Mar 31 16:10:23.359 INFO Generating new mavryk identity. This will take a while, expected_pow: 26
...
```

## Download Tezos Client Binaries (or Build from Source)

Download `mavryk-client`, `mavryk-baker-013-PtJakart` and `mavryk-accuser-013-PtJakart` from the [Tezos project v13.0 Releases page at Gitlab.com](https://gitlab.com/mavryk/mavryk/-/releases). E.g. this [archive containing all x86-64 binaries for v13.0 release](https://gitlab.com/mavryk/mavryk/-/package_files/36986880/download).

Or alternatively, build them from source. See [https://protocol.mavryk.org/introduction/howtoget.html#building-from-sources-via-opam](https://protocol.mavryk.org/introduction/howtoget.html#building-from-sources-via-opam) for instructions.

After successfull compilation, you should see this binaries in Tezos source directory:
```
mavryk-accuser-013-PtJakart
mavryk-baker-013-PtJakart
mavryk-client
```

To prevent `mavryk-client` from issuing a warning about testnet, you can set the variable `MAVRYK_CLIENT_UNSAFE_DISABLE_DISCLAIMER` to `y`.

``` sh
$ export MAVRYK_CLIENT_UNSAFE_DISABLE_DISCLAIMER=y
```
## TezEdge Baker

It is recommended to use TezEdge baker instead of `mavryk-baker-013-PtJakart`.

See [TezEdge Baker](../../apps/baker/README.md#running) to build and run it.


## Wait for TezEdge Node to Sync with the Network

```
$ mavryk-client -E http://localhost:12535 bootstrapped
Waiting for the node to be bootstrapped...
Current head: BLU4di1EGgkd (timestamp: 2021-11-05T23:19:31.000-00:00, validation: 2022-02-24T17:07:41.976-00:00)
Current head: BLKsPJN9yqs9 (timestamp: 2021-11-05T23:19:46.000-00:00, validation: 2022-02-24T17:07:42.100-00:00)
...
Current head: BLMa76HNwT1C (timestamp: 2022-02-24T19:02:54.000-00:00, validation: 2022-02-24T19:03:04.047-00:00)
Node is bootstrapped
```

## Initialize Keys

_TezEdge node have to be synced already._
```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
    activate account my_delegate with "$HOME/data-dir-013-PtJakart/faucet.json"
...
Account my_delegate (tz1XXXXXX) activated with ꜩ76351.572618.
...
```

## Register Baker/Endorser Delegate

_TezEdge node have to be synced already._

```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
    register key my_delegate as delegate
...
The operation has only been included 0 blocks ago.
We recommend to wait more.
...
```

## Run Baker

_Note that the delegate needs to have at least ꜩ6,000 (own or delegated funds) to get baking/endorsing rights._

_Also it takes several cycles to get baking/endorsing rights (2 + num of preserved cycles)_

### TezEdge Baker

```
$ tezedge-baker \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   --baker my_delegate
```


### Mavkit Baker

_Note. For Tezos baker executable from v12.x.x and higher `--media-type json` (or `-m json`) paramters should be added to make it expect JSON RPC instead of new compact encoding_

```
$ mavryk-baker-013-PtJakart \
   --endpoint "http://localhost:12535" \
   --media-type json \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   run with local node "$HOME/data-dir-013-PtJakart/tezedge-data" my_delegate
Node is bootstrapped.
Waiting for protocol 013-PtJakart to start...
Baker v13.0 (cb9f439e) for PtJakart2xVj started.
May 30 16:13:47.527 - 013-PtJakart.baker.transitions: received new head ... at
May 30 16:13:47.527 - 013-PtJakart.baker.transitions:   level 614179, round 0
May 30 16:14:01.447 - 013-PtJakart.baker.transitions: received new head ... at
May 30 16:14:01.447 - 013-PtJakart.baker.transitions:   level 614180, round 0
...
```



## Run accuser

```
$ mavryk-accuser-013-PtJakart \
   --endpoint "http://localhost:12535" \
   --media-type json \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   run
Node is bootstrapped.
Accuser v13.0 (cb9f439e) for PtJakart2xVj started.
May 30 16:15:02.443 - 013-PtJakart.delegate.denunciation: block ... registered
...
```

# Use Ledger

It is possible to use hardware wallet to securely store your Tezos accounts. Currently Ledger Nano S is supported. Using Ledger Live desktop application is the easiest way to manage your Ledger Nano S.

## Initialize Ledger Nano S

You need to fully initialize your device. For details see here: http://ledger.com/start.

## Install Tezos Wallet and Tezos Baker

_To install Tezos Baker, developer mode should be enabled in Ledger Live_

For extensively detailed information about these applications, visit the developer's [GitHub page](https://github.com/obsidiansystems/ledger-app-mavryk).

## Create New Account

If you do not have a Tezos account in your ledger, you need to create a new one. _Note that you should not use your mainnet accounts within a testnet_.

You can create a new Tezos account using Ledger Live application.

## Import Ledger Account to Tezos Client

For this and the following steps, `mavryk-client` executable will be used.

First, inspect if the ledger is recognized by the `mavryk-client`

```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   list connected ledgers
## Ledger `major-squirrel-thick-hedgehog`
Found a Tezos Wallet 2.1.0 (git-description: "091e74e9") application running
on Ledger Nano S at
[IOService:/AppleACPIPlatformExpert/PCI0@0/AppleACPIPCI/XHC1@14/XHC1@14000000/HS03@14300000/Nano
S@14300000/Nano S@0/IOUSBHostHIDDevice@14300000,0].

To use keys at BIP32 path m/44'/1729'/0'/0' (default Tezos key path), use one
of:

mavryk-client import secret key ledger_username "ledger://major-squirrel-thick-hedgehog/bip25519/0h/0h"
mavryk-client import secret key ledger_username "ledger://major-squirrel-thick-hedgehog/ed25519/0h/0h"
mavryk-client import secret key ledger_username "ledger://major-squirrel-thick-hedgehog/secp256k1/0h/0h"
mavryk-client import secret key ledger_username "ledger://major-squirrel-thick-hedgehog/P-256/0h/0h"
```

Use the second proposed command (with `ed25519` curve) to import public key from the ledger. _Note that despite the command name is `import secret key`, this is only public key that is imported._

```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   import secret key my_delegate "ledger://major-squirrel-thick-hedgehog/ed25519/0h/0h"
```

You will need to confirm addition of the address on the ledger.

Check that the address is now known to the `mavryk-client`.

```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   list known addresses
my_delegate: tzXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX (ledger sk known)
```

## Register New Account as a Delegate

To get baking/endorsing rights, this account needs owning some funds, at least ꜩ6,000 (one roll). One way of getting them is to receive them from a faucet account (see [Get free XTZ](#get-free-xtz)).

```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
    activate account faucet with "$HOME/data-dir-013-PtJakart/faucet.json"
...
Account faucet (tz1XXXXXX) activated with ꜩ76351.572618.
...
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   transfer 76351.572618 from faucet to my_delegate
...
```

Now register this account in the network as a Tezos delegate:

```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   register key my_delegate as delegate
```


## Run Baker/Endorser

Start baker daemon:

```
$ mavryk-baker-013-PtJakart \
   --endpoint "http://localhost:12535" \
   --media-type json \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   run with local node "$HOME/data-dir-013-PtJakart/tezedge-data" my_delegate
Node is bootstrapped.
Waiting for protocol 013-PtJakart to start...
Baker v13.0 (cb9f439e) for PtJakart2xVj started.
May 30 19:09:49.898 - 013-PtJakart.baker.transitions: received new head ... at
May 30 19:09:49.898 - 013-PtJakart.baker.transitions:   level 614596, round 0
...
```

## Set Up the Ledger for Baking

Usually you need to interact with ledger to confirm a signing operation. For baking/endorsing Tezos Baking application offers automated signing, limited to blocks and endorsements only.

To enable non-interactive singing of blocks and endorsements use the following command:

```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   setup ledger to bake for my_delegate
```

Make sure you confirm this operation with your ledger.

To make ledger sign blocks and endorsements non-interactively, you need to make _Tezos Baker_ application active on it.

# Remote Signing

To decouple the node processing blocks and operations and keys used to sign them _remote singer_ can be used. In this case the `mavryk-baker` daemon along with a Tezedge node might be running on a cloud server, and communicating with `mavryk-signer` application running on a home server with Ledger Nano S connected to it.

Tezos client binaries can communicate with `mavryk-signer` process via different transports, like _http_, _https_, _tcp_ and _unix_ for unix domain socket. Also it is possible to set up authorization so it accepts signing requests only from authorized client. For more information see [Signer](http://protocol.mavryk.org/user/key-management.html#signer) in Tezos documentation.

## Set up Signing Server

Make sure that your wallet is available for Tezos client applications on the signing server. If not, you should use commands mentioned above in [Importing Ledger Account to Tezos Client](#importing-ledger-account-to-mavryk-client)

Start remote signer application with schema of your choice. For example, the following command will start singning server that uses HTTP transport and listens for incoming connections on port 12536.

```
$ mavryk-signer \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   launch http signer --address 0.0.0.0 -p 12536
Feb 25 18:09:09.074 - signer.http: accepting HTTP requests on port 12536
Feb 25 18:09:09.074 - signer.http: listening on address: ::ffff:0.0.0.0
```

_Note that the Ledger Nano S should be running Tezos Baking application so `mavryk-signer` can use the baker account stored there_

## Run Tezedge Node and Tezos Baker with Remote Signer

Start the Tezedge node:

To make `mavryk-baker` use remote signing, corresponding remote address should be added to Tezos wallet. If the home server from above is accessible via name `home`, you can use the following command:

```
$ mavryk-client \
   --endpoint "http://localhost:12535" \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   import secret key my_delegate http://home:12536/tz1XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

Now you can run `mavryk-baker` using this alias:

```
$ mavryk-baker-013-PtJakart \
   --endpoint "http://localhost:12535" \
   --media-type json \
   --base-dir "$HOME/data-dir-013-PtJakart/client" \
   run with local node "$HOME/data-dir-013-PtJakart/tezedge-data" my_delegate
Node is bootstrapped.
Waiting for protocol 013-PtJakart to start...
Baker v13.0 (cb9f439e) for PtJakart2xVj started.
May 30 19:12:54.408 - 013-PtJakart.baker.transitions: received new head ... at
May 30 19:12:54.408 - 013-PtJakart.baker.transitions:   level 614606, round 1
...
```
