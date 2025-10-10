
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
journalctl -b | grep 'avc:  denied'

# Put SELinux in permissive mode
sudo setenforce 0
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

## <ins>Key Features</ins> 🌟 <a name="features"></a>

### **Blockchain Core**
- **Block Validation**: Comprehensive structural and content validation
- **UTXO Management**: Unspent transaction output tracking
- **Transaction Processing**: Input/output validation and balance verification
- **Chain Synchronization**: P2P block synchronization with peers

### **P2P Networking**
- **libp2p Integration**: Modern peer-to-peer networking stack
- **Protocol Support**: Custom blockchain protocol over request-response
- **Peer Discovery**: Automatic peer discovery and connection management
- **Event-Driven**: Asynchronous network event handling

### **Persistent Storage**
- **Sled Database**: Embedded key-value storage for blockchain data
- **ACID Transactions**: Reliable data consistency guarantees
- **Outbox Pattern**: Reliable event publishing with at-least-once delivery
- **Efficient Indexing**: Height-based and hash-based block lookups

---

## <ins>Architecture Overview</ins> 🏗️ <a name="architecture"></a>

The codebase leverages **Domain-Driven Design (DDD)** patterns and **Clean Architecture (CA)** principles to ensure maintainability, testability and separation of concerns.

This project is organized into **4 distinct layers**, each with clear responsibilities and enforced boundaries:

<details>

<summary>🔎 <ins><em>Tell me more...</em></ins></summary>

### 🟣 <ins>**Domain Layer**</ins> (`domain`)
The core business logic layer containing:
- **Entities**: Core business objects with identity
- **Value Objects**: Immutable types representing domain concepts
- **Repository Traits**: Abstract contracts for domain-level data persistence
- **Domain Validation**: Business rule enforcement at the entity level
- **System Abstractions**: Blockchain, UTXO, Network etc

### 🔵 <ins>**Application Layer**</ins> (`application`)
Orchestrates blockchain workflows without implementation details:
- **Use Cases**: Application-specific business logic
- **Application Services**: Cross-cutting concerns (authentication, authorization)
- **Queue Management**: Orchestrators for async tasks
- **Outbox Relay**: Reliable event publishing for atomic operations
- ~~**Repository Traits**: Abstract contracts for app-level data persistence~~ (none yet)
- **Application DTOs**: Data transfer objects for interlayer communication

### 🟢 <ins>**Infrastructure Layer**</ins> (`infrastructure`)
Concrete implementations of abstract contracts:
- **Repository Implementations**: **Sled**-based blockchain data persistence
- **Network Protocol**: **libp2p**-based P2P networking
- **Unit of Work**: Atomic transactions
- **External Service Adapters**: JWT handling, password hashing
- **Infrastructure DTOs**: Storage-specific data models

### 🟡 <ins>**Presentation Layer**</ins> (`presentation`)
HTTP API and external interfaces:
- **HTTP Handlers**: REST endpoints for blockchain queries
- **DTOs**: API request/response models
- **Authentication Extractors**: JWT token validation
- **OpenAPI Documentation**: Auto-generated via **utoipa**

### 📦 <ins>**Supporting Crates**</ins>

#### <ins>**Common**</ins> (`common`)
Shared utilities across all layers:
- **Logging**: Structured logging macros
- **Error Types**: Standardized blockchain error handling
- **Configuration**: Configuration data types
- **Transaction Abstractions**: Infrastructure-agnostic transaction management (allows for CA-compliant use cases)
- **Cross-cutting Utilities**: Shared types and helper functions

#### <ins>**Main**</ins> (`main`)
Application entry point and dependency injection:
- **Blockchain Node Startup**: P2P blockchain node bootstrapping
- **HTTP Server Startup**: HTTP server initialization and middleware setup
- **Environment Setup**: Configuration loading and validation
- **Dependency Wiring**: Service registration and dependency injection

#### <ins>**Macros**</ins> (`macros`)
Custom procedural macros for code generation:
- **Logging Macros**: Configurable logging macro generator

### <ins>Clean Architecture Dependency Flow</ins>

The layers follow strict dependency rules to maintain clean architecture:

- **Domain** depends on nothing (pure blockchain logic)
- **Application** depends solely on *Domain*
- **Infrastructure** depends on *Domain* and *Application*
- **Presentation** depends on *Application* and *Domain*
- **Common** is dependency-free and accessible by all layers
- **Macros** is exclusively used by *Common*
- **Main** depends on all layers to wire everything together

</details>

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
