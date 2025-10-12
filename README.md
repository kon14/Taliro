
<div align="center">
<br>
<a href="https://github.com/kon14/Taliro" target="_blank">
    <h1>Taliro 🪙</h1>
</a>
A UTXO-based blockchain P2P node implementation written in <strong>Rust</strong> 🦀.
</div>

<hr />

Despite its simplicity, this project aims to demonstrate a ~~fully~~ somewhat-functional UTXO-based blockchain node.<br />
Taliro nodes connect to each other over a P2P network, exchanging blocks and transactions.<br />
Each node maintains its own local copy of the blockchain.

A RESTful API is exposed for **developer-focused** interaction and inspection purposes.<br />
It allows for basic blockchain state querying, submission of transactions, ad hoc block mining and peer inspection.<br />
This is **NOT** meant to be a secure user-facing API, as such endpoint payloads are structured in a way that facilitates development.<br />
Rudimentary dev endpoint authorization is supported via an optional master key secret.

---

## <ins>Running via Docker Compose</ins> 💻 <a name="run-compose"></a>

``` bash
# Bring up two Taliro nodes (no authorization)
> docker compose up --build

# Bring up two Taliro nodes (secret key authorization)
> HTTP_MASTER_KEY_SECRET="7h3 c4k3 15 4 l13" docker compose up --build

# View node logs
> docker compose logs -f taliro-node-alpha
> docker compose logs -f taliro-node-beta

# Stop the nodes
> docker compose down

# Stop and remove all data
> docker compose down -v
```

<details>

<summary>🔐 <ins><em>SELinux Compatibility</em></ins></summary>

SELinux users may face issues accessing the Docker socket, despite being members of the `docker` user group.<br />
While configuring SELinux policies is outside the scope of this readme file, you may temporarily bypass it as follows:

``` bash
# Check for AVC denial logs
> journalctl -b | grep 'avc:  denied'

# Put SELinux in permissive mode
> sudo setenforce 0
```

</details>

<details>

<summary>🦭 <ins><em>Podman Compatibility</em></ins></summary>

Podman is technically supported, but depending on your setup, you might need a couple of workarounds.<br />
Head over to `docker-compose.yml` and flip any relevant `promtail` volume entries to their corresponding podman variants.

If you're facing issues around DNS resolution, you're likely hitting a `podman-compose` bug resulting in containers not being registered to the `taliro-net` network.<br />
Troubleshooting instructions are provided within `docker-compose.yml`.

</details>


``` bash
# Navigate to Swagger UI (on Linux)
> xdg-open "http://localhost:4100/swagger/index.html" # Node Alpha
> xdg-open "http://localhost:4200/swagger/index.html" # Node Beta
```

Note: Make sure you bring up the full Swagger UI path to avoid 404s caused by path normalization redirects.

---

## <ins>Monitoring</ins> 🕵🏻 <a name="monitoring"></a>

``` bash
# Navigate to Grafana (on Linux)
> xdg-open "http://localhost:3000"
```

Note: Grafana might take a few seconds to spin up.<br />
Credentials: `admin` / `admin`.

---

## <ins>Architecture Overview</ins> 🏗️ <a name="architecture"></a>

``` mermaid
flowchart TD
    HTTP["<ins>**HTTP Dev API**</ins><br />(Axum)"]
    P2P["<ins>**P2P Network**</ins><br />(libp2p)<br /><br />- Gossipsub<br />- Kademlia<br />- Taliro (Request/Response)"]
    UC[Application Use Cases]
    NODE_CMD[<ins>**Node**</ins><br />Command Orchestration<br />Event Loop]
    BC[Blockchain]
    MP[Mempool]
    UTXO[UTXO Set]
    BLOCK_VAL[Block Validator]
    TX_VAL[Transaction Validator]
    SYNC_Q[Block Sync Queue]
    PROC_Q[Block Processing Queue]
    OUTBOX[Outbox]
    SLED["<ins>**Storage**</ins><br />(Sled)<br />"]
    
    HTTP --> UC
    P2P --> NODE_CMD
    UC --> NODE_CMD
    NODE_CMD --> BC
    NODE_CMD --> MP
    NODE_CMD --> UTXO
    NODE_CMD --> P2P
    NODE_CMD --> SYNC_Q
    SYNC_Q --> PROC_Q
    BC --> BLOCK_VAL
    BC --> OUTBOX
    BLOCK_VAL --> TX_VAL
    MP --> TX_VAL
    OUTBOX --> NODE_CMD
    BC --> SLED
    UTXO --> SLED
    NODE_CMD --> SLED
    OUTBOX --> SLED
    
    %% Styling External Boundaries
    style HTTP fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000
    style P2P fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000
    
    %% Styling Node Orchestrator
    style NODE_CMD fill:#87ceeb,stroke:#000000,stroke-width:3px,color:#000000
    
    %% Styling Core Components
    style BC fill:#b19cd9,stroke:#000000,stroke-width:2px,color:#000000
    style MP fill:#b19cd9,stroke:#000000,stroke-width:2px,color:#000000
    style UTXO fill:#b19cd9,stroke:#000000,stroke-width:2px,color:#000000
    
    %% Styling Internal Storage
    style SLED fill:#90ee90,stroke:#000000,stroke-width:2px,color:#000000
```

I see you've taken an interest in this one 🤔.<br />
Whether it's cause you like what you see or you absolutely loathe it...<br />
You may always refer to the [Architecture Documentation](./ARCHITECTURE.md) for a detailed analysis of the project and its design decisions.

---

## <ins>Environment Variables</ins> 📃 <a name="env-vars"></a>


|          Variable           | Description                                                                                                                                                                                                                                                            | Required  |       Default        |                                            Example                                            |
|:---------------------------:|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|:---------:|:--------------------:|:---------------------------------------------------------------------------------------------:|
|      `STORAGE_DB_PATH`      | The filesystem path to be used for your `Sled` storage.                                                                                                                                                                                                                |  `True`   |          —           |                               `$XDG_CONFIG_HOME/blockchain/db`                                |
|       `HTTP_API_PORT`       | The port to be used by the HTTP server.                                                                                                                                                                                                                                |  `False`  |        `4000`        |                                            `8080`                                             |
|     `HTTP_API_BASE_URL`     | A public URL pointing to the backend API's root path.                                                                                                                                                                                                                  |  `True`   |          —           |                                   `https://foo.bar.baz/api`                                   |
|  `HTTP_MASTER_KEY_SECRET`   | Optional secret to be used for development endpoint authorization.                                                                                                                                                                                                     |  `False`  |          —           |                                      `7h3 c4k3 15 4 l13`                                      |
|  `NETWORK_LISTEN_ADDRESS`   | The P2P node's network address.<br />Using an epheral port (`tcp/0`) will prohibit peers from reconnecting on restart.                                                                                                                                                 |  `False`  | `/ip4/0.0.0.0/tcp/0` |                                `/ip4/192.168.1.125/tcp/54244`                                 |
|    `NETWORK_INIT_PEERS`     | Semicolon-separated list of initial peer addresses (multiaddr with peer id).                                                                                                                                                                                           |  `False`  |          —           |    `/ip4/192.168.1.125/tcp/54244/p2p/12D3KooWSg4ox9udRcwrjo8ETg1gjB7g5wSSwjVMGKWJiqF9XjdB;`   |
| `NETWORK_IDENTITY_KEY_PAIR` | `Base64` encoded `ed25519` key pair to be used for persistent node identity.<br />May be obtained from the dev HTTP API (`GET @ /dev/network/self`).                                                                                                                   |  `False`  |      Generated       | `CAESQPDur8zTyaDoZwmCIhtpdaE5s-TjOZd8iQhHKaaL7hQ6-nZnaha4CWVWEtIfYx4Vx53sxrChvlm25_EhXftu9Yo` |
|         `RUST_LOG`          | Specifies the desired logging level.<br />Refer to the [tracing_subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#method.from_default_env) documentation for details.<br />Syntax is [env_logger](https://docs.rs/env_logger/latest/env_logger/)-compatible. |  `False`  |       `error`        |                                            `info`                                             |
|        `CONFIG_PATH`        | Optional path to a `TOML` configuration file.                                                                                                                                                                                                                          |  `False`  |          —           |                           `$XDG_CONFIG_HOME/blockchain/config.toml`                           |

---

## <ins>Container Health Monitoring</ins> 🏥 <a name="health-checks"></a>

The container image includes built-in health checks:

``` bash
# Check container health status
> docker ps --filter "name=taliro-node"

# View health check logs
> docker inspect --format='{{json .State.Health}}' taliro-node-alpha
> docker inspect --format='{{json .State.Health}}' taliro-node-beta
```

<details>

<summary>🦭 <ins><em>Podman Compatibility</em></ins></summary>

Podman image builds default to the `oci` format, which doesn't support health check instructions.<br />
Make sure you build your images in `docker` format to retain health check functionality.

``` bash
# Build image, preserving health check instructions
> podman build -t taliro --format docker .
```

</details>

---

## <ins>Local Development</ins> 👨🏻‍🔬 <a name="local-dev"></a>

The following section assumes your environment contains an installation of the [Rust development toolchain](https://www.rust-lang.org/tools/install).

``` bash
# Prepare Git Hooks (Optional)
lefthook install
```

``` bash
# Build
> cargo build

# Bring up a node
> HTTP_API_BASE_URL=http://localhost:4000 \
HTTP_API_PORT=4000 \
HTTP_MASTER_KEY_SECRET="7h3 c4k3 15 4 l13" \
NETWORK_LISTEN_ADDRESS=/ip4/0.0.0.0/tcp/2048 \
NETWORK_INIT_PEERS=/ip4/192.168.1.125/tcp/2049/p2p/12D3KooWKwUzXLNEAF97yuvyvWNVVunxAULArPj7pHWAvSveU1rc; \
NETWORK_IDENTITY_KEY_PAIR=CAESQPDur8zTyaDoZwmCIhtpdaE5s-TjOZd8iQhHKaaL7hQ6-nZnaha4CWVWEtIfYx4Vx53sxrChvlm25_EhXftu9Yo \
cargo run
```
