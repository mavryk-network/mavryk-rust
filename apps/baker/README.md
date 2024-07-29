# TezEdge Baker

This is a daemon for creating, signing and injecting endorsements and blocks for the Tezos blockchain, in accordance to the Tenderbake consensus algorithm.

## Running

### Prerequisites

|  OS  |      Versions      |
|----------|:-------------:|
| Ubuntu |  16.04, 18.04, 18.10, 19.04, 19.10, 20.04, 20.10, 21.04, 21.10, 22.04 |
| Debian |  9, 10 |
| OpenSUSE |  15.1, 15.2 |
| CentOS |  8 |
| MacOS |  *experimental* - newer or equal to 10.13 should work, Intel and M1 cpus |

1. Install **Git** (client), **Curl**, a several other tools and libs in your OS. In Ubuntu and Debian it will be:
    ```
    sudo apt install git curl libssl-dev clang gcc pkg-config make libsodium-dev libev-dev
    ```
2. Install **Rust** command _(We recommend installing Rust through rustup.)_
    ```
    # Run this in your terminal, then follow the onscreen instructions.
    curl https://sh.rustup.rs -sSf | sh
    source ~/.cargo/env
    ```
3. Install **Rust toolchain** _(Our releases are built with 1.58.1.)_
    ```
    rustup toolchain install 1.58.1
    rustup default 1.58.1
    ```
4. Install **Tezos Client** and **Tezos Signer**
    ```
    curl https://gitlab.com/mavryk/mavryk/-/package_files/36986666/download -o mavryk-client
    chmod +x mavryk-client
    sudo mv mavryk-client /usr/local/bin
    curl https://gitlab.com/mavryk/mavryk/-/package_files/36986689/download -o mavryk-signer
    chmod +x mavryk-signer
    sudo mv mavryk-signer /usr/local/bin
    ```

### Building

#### Option 1

Use the tool cargo, which is part of **Rust** already installed in the previous step:
```
cargo install --git https://github.com/tezedge/tezedge --tag v3.1.1 baker
```

You can install the latest development version using `--branch develop` instead of `--tag v2.3.x`.

```
cargo install --git https://github.com/tezedge/tezedge --branch develop baker
```

The cargo will create the binary `tezedge-baker` at `~/.cargo/bin`. The directory `~/.cargo/bin` should be in your `$PATH` variable, so you can run `tezedge-baker` without the full path. If it is not working (it might not work if you are using a non-standard shell), execute `source ~/.cargo/env` to update the environment.

#### Option 2

Clone sources and build:
```
git clone https://github.com/tezedge/tezedge
cd tezedge
git checkout tags/v3.1.1 -b v3.1.1
cargo build -p baker --release
```

The binary will be in `target/release`.

Make sure you copy the binary into one of the directories listed in `$PATH` variable, for example:

```
sudo cp target/release/tezedge-baker /usr/local/bin
```

### Run the TezEdge node

Get the source code, and build the node

```
git clone https://github.com/tezedge/tezedge
cd tezedge
git checkout tags/v3.1.1 -b v3.1.1
cargo build -p light-node -p protocol-runner --release
```

Open the page [Snapshots](http://snapshots.tezedge.com), choose the network either mainnet or testnet. Choose Storage: tezedge, and Type: Full.

![Snapshot page](docs/screenshot-snapshot-page.png)

You will see blocks, each block is a snapshot, the top-most block is most recent snapshot. Every day, a new link to the snapshot will appear. Copy the link.

![Link](docs/screenshot-snapshot-link.png)

To run the node on testnet use the following commands, replace `...` with the actual link copied on previous step:
```
mkdir -p "$HOME/data-ithacanet/tezedge-data"
./target/release/light-node import-snapshot \
    --from ... \
    --mavryk-data-dir "$HOME/data-ithacanet/tezedge-data"
KEEP_DATA=1 ./run.sh release \
    --network=ithacanet \
    --mavryk-data-dir "$HOME/data-ithacanet/tezedge-data" \
    --mavryk-context-storage=tezedge
```

To run the node on mainnet, use the following commands:
```
mkdir -p "$HOME/data-mainnet/tezedge-data"
./target/release/light-node import-snapshot \
    --from ... \
    --mavryk-data-dir "$HOME/data-mainnet/tezedge-data"
KEEP_DATA=1 ./run.sh release \
    --network=mainnet \
    --mavryk-data-dir "$HOME/data-mainnet/tezedge-data" \
    --mavryk-context-storage=tezedge
```

The node will be building for some time and then it will run. Keep it running. Continue in another terminal tab.

### Prepare an account

Skip this section if you already have a Tezos account ready for baking.

Pick a new <delegate_alias> and generate a new Tezos account using the Mavkit client.

```
mavryk-client -E "http://localhost:18732" gen keys <delegate_alias>
```

You need to fund this account with at least 6000 ꜩ. Register the account as a delegate and wait the amount of time equal to between 5 and 6 cycles, depending on the position in the cycle (approximately 15 days).

```
mavryk-client -E "http://localhost:18732" register key <delegate_alias> as delegate
```

By default, mavryk-client stores the secret key for the account in the `$HOME/.mavryk-client` directory.

See the [baking documentation](../../baking/mainnet/README.md#initialize-keys-for-bakerendorser) for more details.

See the [Key management](https://protocol.mavryk.org/user/key-management.html) guide for more information.

### Use ledger

Alternatively, you can use an external ledger.

Install mavryk baker app in ledger by using ledger-live gui tool. You also need to open baker app and enable baking.

Run signer, it will be running in background:
```
nohup mavryk-signer \
    -E "http://localhost:18732" \
    launch http signer &
```

After that, run this in the terminal
```
mavryk-signer \
    -E "http://localhost:18732" \
    list connected ledgers
```

It will print something like this:
```
To use keys at BIP32 path m/44'/1729'/0'/0' (default Tezos key path), use one
of:
  mavryk-client import secret key ledger0 "ledger://reckless-duck-mysterious-wallaby/bip25519/0h/0h"
  mavryk-client import secret key ledger0 "ledger://reckless-duck-mysterious-wallaby/ed25519/0h/0h"
  mavryk-client import secret key ledger0 "ledger://reckless-duck-mysterious-wallaby/secp256k1/0h/0h"
  mavryk-client import secret key ledger0 "ledger://reckless-duck-mysterious-wallaby/P-256/0h/0h"
```

User must choose ed25519 link and run:
```
mavryk-signer \
    -E "http://localhost:18732" \
    import secret key <delegate_alias> "ledger://reckless-duck-mysterious-wallaby/ed25519/0h/0h"
```

This will print `added: tz1...`, it is your public key. Run the following command to import it. The command has `secret key` words, but it is working with the link that contains public key hash, the real secret key is still inside the ledger, and isn't exposed.
```
mavryk-client
    -E "http://localhost:18732" \
    import secret key <delegate_alias> http://localhost:6732/tz1...
```

And finally, the Tezos Baking application on the Ledger should be configured for baking:
```
mavryk-client \
    -E "http://localhost:18732" \
    setup ledger to bake for <delegate_alias>
```

If you did not done it before, you need to fund this account with at least 6000 ꜩ. Register the account as delegate and wait the amount of time equal to between 5 and 6 cycles, depending on the position in the cycle (approximately 15 days).

```
mavryk-client -E "http://localhost:18732" register key <delegate_alias> as delegate
```

### Run the baker

_Note: It is recommended to run the baker with nohup_

Assuming that the secret key (or locator for remote signing) is in `$HOME/.mavryk-client` and the node is running locally and uses 18732 port for RPC, the command is:

```
nohup tezedge-baker --base-dir "$HOME/.mavryk-client" --endpoint "http://localhost:18732" --baker <delegate_alias> &
```

Additionally, you can run `tezedge-baker --help` to get short help.

Options:

- `--base-dir`: The base directory. The path to the directory where the baker can find secret keys, or the remote signer's location. Usually, it is `~/.mavryk-client`. Also, this directory is used by baker as a persistent storage of the state. It is crucial, for example, when revealing the seed nonce in a new cycle.
- `--endpoint`: TezEdge or Tezos node RPC endpoint. Usually the port is `8732` or `18732`. If node is running locally, it will be `http://localhost:8732`.
- `--baker`: The alias of the baker.
- `--archive` or `-a`: If this flag is used, the baker will store verbose information for debug in the base directory.

### Common problems

1. After creating an account and registering it as a delegate, make sure you wait at least 5 cycles (approximately 15 days) before you start to bake.
1. The executable `tezedge-baker` and `mavryk-client` should be in the directory which is in the `$PATH` environment variable. It may be `/usr/local/bin`, you need superuser permission to copy in this directory. Also, it is possible to execute `mavryk-client` from any directory, but you need to specify the full path, for example `/home/username/mavryk/mavryk-client gen keys bob`, instead of `mavryk-client gen keys bob`.

## Tests

Run from the source code directory:

```
cargo test -p baker
```

### Fuzzing

Install Rust nightly-2021-12-22 and cargo-fuzzcheck from source.

```
rustup install nightly-2021-12-22
cargo +nightly-2021-12-22 install --git https://github.com/tezedge/fuzzcheck-rs cargo-fuzzcheck
```

Run it from the directory `apps/baker`:

```
cargo +nightly-2021-12-22 fuzzcheck --test action_fuzz test_baker
```

### Mitten tests

Prepare Tezos with mitten.

```
git clone https://gitlab.com/nomadic-labs/mavryk.git mavryk-mitten -b mitten-ithaca
cd mavryk-mitten
opam init --disable-sandboxing
make build-deps
eval $(opam env)
make
make mitten
```

Rename the file `mavryk-mitten/mavryk-baker-012-Psithaca` to `mavryk-mitten/mavryk-baker-012-Psithaca.mavkit`

Copy the files `tezedge/apps/baker/tezedge.env` and `tezedge/apps/baker/mavryk-baker-012-Psithaca` into the `mavryk-mitten` directory.

Build the baker and copy the binary into `mavryk-mitten`.

From the `tezedge` directory:
```
cargo build -p baker --release
cp target/release/tezedge-baker ../mavryk-mitten/mavryk-baker-012-Psithaca.tezedge
```

Now you can run the mitten scenario:

From the `mavryk-mitten` directory:
```
dune exec src/mitten/scenarios/no_eqc_stuck.exe
```

You can find more scenarios in `src/mitten/scenarios`.
