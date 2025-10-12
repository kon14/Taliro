# <ins>Taliro Architecture</ins> 🏗️

This document provides an in-depth overview of Taliro's architecture, design patterns and operational flows.

---

## <ins>Table of Contents</ins>

- [Overview](#overview)
- [Architectural Principles](#architectural-principles)
- [Node Lifecycle](#node-lifecycle)
- [Command Pattern & Event Loop](#command-pattern-event-loop)
- [Event Sources & Data Flows](#event-sources-data-flows)
- [Persistence Layer](#persistence-layer)
- [Subsystem Deep Dives](#subsystem-deep-dives)
- [Network Architecture](#network-architecture)
- [Concurrency Model](#concurrency-model)
- [Transaction Guarantees](#transaction-guarantees)
- [Error Handling Strategy](#error-handling-strategy)

---

## <ins>Overview</ins> 🗺️ <a name="overview"></a>

Taliro is a UTXO-based blockchain implementation demonstrating clean architecture principles in a distributed systems context.<br />
The system is composed of multiple cooperating subsystems communicating through a centralized command pattern, ensuring sequential consistency and avoiding race conditions.

### High-Level Architecture <a name="overview--high-level-architecture"></a>

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

---

## <ins>Architectural Principles</ins> 📐 <a name="architectural-principles"></a>

### 🫧 <ins>Clean Architecture & Domain-Driven Design</ins> <a name="architectural-principles--ca-ddd"></a>

The codebase is organized into distinct layers with strict dependency rules, ensuring maintainability, testability and separation of concerns.

- **Domain Layer**: Pure business logic with no external dependencies
- **Application Layer**: Use cases and workflow orchestration
- **Infrastructure Layer**: Concrete implementations (storage, networking)
- **Presentation Layer**: External interfaces (HTTP API)

<details>

<summary>🔎 <ins><em>Tell me more...</em></ins></summary>

#### 🟣 <ins>**Domain Layer**</ins> (`domain`)
The core business logic layer containing:
- **Entities**: Core business objects with identity
- **Value Objects**: Immutable types representing domain concepts
- **Repository Traits**: Abstract contracts for domain-level data persistence
- **Domain Validation**: Business rule enforcement at the entity level
- **System Abstractions**: Blockchain, UTXO, Network etc

#### 🔵 <ins>**Application Layer**</ins> (`application`)
Orchestrates blockchain workflows without implementation details:
- **Use Cases**: Application-specific business logic
- **Application Services**: Cross-cutting concerns (authentication, authorization)
- **Queue Management**: Orchestrators for async tasks
- **Outbox Relay**: Reliable event publishing for atomic operations
- ~~**Repository Traits**: Abstract contracts for app-level data persistence~~ (none yet)
- **Application DTOs**: Data transfer objects for interlayer communication

#### 🟢 <ins>**Infrastructure Layer**</ins> (`infrastructure`)
Concrete implementations of abstract contracts:
- **Repository Implementations**: **Sled**-based blockchain data persistence
- **Network Protocol**: **libp2p**-based P2P networking
- **Unit of Work**: Atomic transactions
- **External Service Adapters**: JWT handling, password hashing
- **Infrastructure DTOs**: Storage-specific data models

#### 🟡 <ins>**Presentation Layer**</ins> (`presentation`)
HTTP API and external interfaces:
- **HTTP Handlers**: REST endpoints for blockchain queries
- **DTOs**: API request/response models
- **Authentication Extractors**: JWT token validation
- **OpenAPI Documentation**: Auto-generated via **utoipa**

#### 📦 <ins>**Supporting Crates**</ins>

##### <ins>**Common**</ins> (`common`)
Shared utilities across all layers:
- **Logging**: Structured logging macros
- **Error Types**: Standardized blockchain error handling
- **Configuration**: Configuration data types
- **Transaction Abstractions**: Infrastructure-agnostic transaction management (allows for CA-compliant use cases)
- **Cross-cutting Utilities**: Shared types and helper functions

##### <ins>**Main**</ins> (`main`)
Application entry point and dependency injection:
- **Blockchain Node Startup**: P2P blockchain node bootstrapping
- **HTTP Server Startup**: HTTP server initialization and middleware setup
- **Environment Setup**: Configuration loading and validation
- **Dependency Wiring**: Service registration and dependency injection

##### <ins>**Macros**</ins> (`macros`)
Custom procedural macros for code generation:
- **Logging Macros**: Configurable logging macro generator

#### <ins>Clean Architecture Dependency Flow</ins>

The layers follow strict dependency rules to maintain clean architecture:

- **Domain** depends on nothing (pure blockchain logic)
- **Application** depends solely on *Domain*
- **Infrastructure** depends on *Domain* and *Application*
- **Presentation** depends on *Application* and *Domain*
- **Common** is dependency-free and accessible by all layers
- **Macros** is exclusively used by *Common*
- **Main** depends on all layers to wire everything together

---

</details>

### 🧬 <ins>Dependency Inversion</ins> <a name="architectural-principles--dependency-inversion"></a>

All layers depend on abstraction traits for cross-layer functionality.<br />
Concrete implementations are injected at the application entry point (`main`).

Internal layer dependencies are also typically abstracted to facilitate testing and modularity.<br />
Their implementations are defined and injected at local crate level.

### 🍳 <ins>Single Responsibility</ins> <a name="architectural-principles--single-responsibility"></a>

Each subsystem has a clearly defined purpose:
- **Node**: Central command loop orchestrating most operations
- **Blockchain**: Block storage and chain state management
- **Mempool**: Pending transaction pool
- **UTXO Set**: Unspent transaction output tracking
- **Network**: P2P communication and peer management
- **Validation**: Block and transaction validation logic

### 🎢 <ins>Sequential Consistency</ins> <a name="architectural-principles--seq-consistency"></a>

All state-mutating operations flow through a single command queue, processed sequentially by the node's event loop.<br />
This eliminates race conditions and simplifies reasoning about system state.

### 🛂 <ins>Type-Enforced Validation</ins> <a name="architectural-principles--validation"></a>

Domain entities are represented by two type-safe variants based on their validation state:
- **Pre-Validation Types** (e.g.`NonValidatedBlock`): Untrusted data from external sources
- **Post-Validation Types** (e.g. `Block`): Cryptographically and structurally verified entities

**Trust Boundaries**:

``` mermaid
flowchart TD
    HTTP["**HTTP Dev API**<br />(Axum)"]
    P2P["**Taliro P2P Network**<br />(libp2p)"]
    STORAGE["**Internal Storage**<br />(Sled)"]
    
    V_P2P["Validated Type<br />(from wire)"]
    NV_COMMON[NonValidated Type]
    VAL[Validator]
    V_COMMON[Validated Type]
    LOGIC[Usage in<br />Domain]

    HTTP --> NV_COMMON
    P2P --> V_P2P
    V_P2P --> NV_COMMON
    NV_COMMON --> VAL
    VAL --> V_COMMON
    STORAGE --> V_COMMON
    V_COMMON --> LOGIC
    
    %% Styling Entry Points
    style HTTP fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000
    style P2P fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000
    style STORAGE fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000

    %% Styling LOGIC node (Usage in Domain)
    style LOGIC fill:#a0c8f0,stroke:#000000,stroke-width:2px,color:#000000
```

<details>

<summary>🔎 <ins><em>Tell me more...</em></ins></summary>

<br />

**Key Principles**:
- **External Data**: Always treated as non-validated, must pass validation before usage
- **Storage Data**: Only persisted after validation, retrieved as validated types (trusted internal source)
- **Zero Trust**: Even data from trusted peers undergoes full validation before acceptance

**Design Benefits**:
- Type system enforces validation requirements at compile time
- Impossible to accidentally use unvalidated data in critical operations
- Clear audit trail of validation boundaries
- Defense against malicious or buggy peers

</details>

---

## 🌱 <ins>Node Lifecycle</ins> <a name="node-lifecycle"></a>

A Taliro node progresses through several states during its lifecycle:

``` mermaid
stateDiagram-v2
    [*] --> Initialized
    Initialized --> Bootstrapped
    Bootstrapped --> Started
    Started --> Running
    Running --> Terminating
    Terminating --> [*]
    
    note right of Initialized
        Core subsystems created
        Repositories wired
    end note
    
    note right of Bootstrapped
        Network connected
        P2P engine online
    end note
    
    note right of Started
        Command handlers configured
        Ready to process
    end note
    
    note right of Running
        Main event loop active
        Processing commands
    end note
    
    note left of Terminating
        Graceful shutdown
        In progress
    end note
```

---

## </ins>Command Pattern & Event Loop</ins> 🔄 <a name="command-pattern-event-loop"></a>

### 🗜️ <ins>Architecture</ins> <a name="command-pattern-event-loop--architecture"></a>

Taliro uses a **command pattern** with a centralized event loop for all node operations.<br />
This provides:
- Sequential consistency (no race conditions)
- Clear audit trail (all operations logged)
- Simplified debugging (single point of execution)
- Backpressure handling (bounded channel)

Commands are categorized by subsystem.

### 🧭 <ins>Command Flow</ins> <a name="command-pattern-event-loop--cmd-flow"></a>

``` mermaid
flowchart TD
    HTTP["**HTTP API**"]
    P2P["**P2P Network**"]
    FACTORY["<code>CommandResponderFactory::build_*_cmd()</code><br /><code>(NodeCommandRequest, Future&lt;Response&gt;)</code>"]
    SENDER["<code>CommandSender::send(command)</code><br />(MPSC channel)"]
    LOOP["<code>NodeRunning</code> Event Loop<br />(Single-threaded sequential processing)"]
    DISPATCHER["<code>CommandDispatcher</code> routes to appropriate handler based on command type"]
    HANDLER["Handler Executes & Responds"]
    RESPONSE["Source awaits <code>Future</code>, processes response"]
    
    HTTP --> FACTORY
    P2P --> FACTORY
    FACTORY --> SENDER
    SENDER --> LOOP
    LOOP --> DISPATCHER
    DISPATCHER --> HANDLER
    HANDLER --> RESPONSE
    
    %% Styling Entry Points
    style HTTP fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000
    style P2P fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000
    
    %% Styling Exit Point
    style RESPONSE fill:#a0c8f0,stroke:#000000,stroke-width:2px,color:#000000
```

[//]: # (### Command Categories)

[//]: # ()
[//]: # (Commands are categorized by subsystem:)

[//]: # ()
[//]: # (- **Blockchain Commands**: Genesis, mining, block append, block queries)

[//]: # (- **Mempool Commands**: Transaction submission, mempool queries)

[//]: # (- **UTXO Commands**: UTXO set queries)

[//]: # (- **Network Commands**: Peer management, network info)

[//]: # (- **P2P Protocol Commands**: Block sync, tip exchange)

[//]: # (- **System Commands**: Node shutdown)

### 📟 <ins>Request-Response Mechanism</ins> <a name="command-pattern-event-loop--req-res"></a>

Each command uses a **oneshot channel** for responses, providing:
- Type-safe responses
- Timeout capability
- Zero-copy response delivery
- Clear ownership semantics

---

## <ins>Event Sources & Data Flows</ins> 🌊  <a name="event-sources-data-flows"></a>

Taliro processes events from **two primary sources**: the HTTP Dev API and the P2P Network.

Both flows converge at the **Node Command Queue**, ensuring:
- **Sequential Processing**: No race conditions between HTTP and P2P events
- **Uniform Handling**: Same validation and state update logic regardless of source
- **Decoupling**: HTTP and P2P layers don't directly interact with subsystems
- **Backpressure**: Bounded channel prevents overwhelming the node

### 👨🏻‍💻 <ins>HTTP-Initiated Flow (Developer API)</ins> <a name="event-sources-data-flows--http"></a>

``` mermaid
flowchart TD
    REQ["**HTTP Request**"]
    HANDLER["**HTTP Handler**<br />(Presentation Layer)<br /><br />- Authenticate (optional master key)<br />- Parse HTTP payload into Presentation Request<br />- Construct Application Request DTO (Domain types)"]
    UC["**Use Case**<br />(Application Layer)<br /><br />- Perform orchestration logic<br />- Create Command(s) via Factory<br />- Dispatch Command(s) through MPSC Channel"]
    CMD["**CommandDispatcher** → **Handler**<br />(Domain)<br /><br />- Validate<br />- Execute operation<br />- Perform state mutations<br />- Respond via Oneshot Channel"]
    UC2["**Use Case**<br />(Application Layer)<br /><br />- Await response future(s)<br />- Construct Application Response DTO"]
    HANDLER2["**HTTP Handler**<br />(Presentation Layer)<br /><br />- Map Application Response DTO to Presentation Response DTO<br />- Serialize to JSON (if applicable)"]
    RESP["**HTTP Response**"]
    
    REQ --> HANDLER
    HANDLER --> UC
    UC --> CMD
    CMD --> UC2
    UC2 --> HANDLER2
    HANDLER2 --> RESP
    
    %% Styling Entry Point
    style REQ fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000
    
    %% Styling Exit Point
    style RESP fill:#a0c8f0,stroke:#000000,stroke-width:2px,color:#000000
```

**Layer Transitions**:
- **Presentation → Application**: HTTP handler extracts request data, maps to Application DTO (domain types), delegates to use case
- **Application → Domain**: Use case orchestrates operation, creates command(s) via factory, sends to node command queue
- **Domain Processing**: Command dispatcher routes to appropriate handler, executes domain logic
- **Domain → Application**: Result returned via oneshot channel, use case awaits and processes response(s)
- **Application Processing**: Use case performs additional orchestration actions
- **Application → Presentation**: Application layer returns Application DTO to HTTP handler
- **Presentation Processing**: HTTP handler maps Application DTO to Presentation DTO, serializes to JSON

**Examples**:
- User submits transaction via HTTP → `PlaceMempoolTransaction` use case → check existing UTXOs via `UtxoCommand::GetUtxosByOutpoints`, build new transaction, store in mempool via `MempoolCommand::PlaceTransaction` command → return transaction
- User requests blockchain tip → `GetBlockchainTipInfo` use case → `BlockchainCommand::GetTipInfo` command → return tip info
- User queries UTXO set → `GetUtxos` use case → `UtxoCommand::GetUtxos` command → return UTXO data

### 🌌 <ins>P2P-Initiated Flow (Network Events)</ins> <a name="event-sources-data-flows--p2p"></a>

``` mermaid
flowchart TD
    EVENT["**P2P Network Event**"]
    LOOP["**Network Event Loop**<br />(Infrastructure)<br /><br />- libp2p swarm processes event<br />- Decode protocol message"]
    FACTORY["**Create Command via Factory**<br /><br />(e.g., HandleReceiveBlocks, ProxyNetworkEvent)"]
    QUEUE["**Send to Node Command Queue**"]
    CMD["**CommandDispatcher** → **Handler**<br /><br />- Process blocks<br />- Update sync queues<br />- Trigger validation"]
    EFFECTS["**Side Effects**<br /><br />- Block processing queue updated<br />- Blockchain state updated<br />- Network broadcasts (if applicable)"]
    
    EVENT --> LOOP
    LOOP --> FACTORY
    FACTORY --> QUEUE
    QUEUE --> CMD
    CMD --> EFFECTS
    
    %% Styling Entry Point
    style EVENT fill:#ffcc00,stroke:#000000,stroke-width:2px,color:#000000
```

**Examples**:
- Peer announces new block (Gossipsub) → decode, validate, process via BlockProcessingQueue
- Peer announces higher tip → `HandleReceiveBlockchainTipInfo` → request missing blocks
- Peer responds to block request → `HandleReceiveBlocks` → queue for processing
- New peer connects → exchange tips, initiate sync if behind

---

## <ins>Persistence Layer</ins> 💾 <a name="persistence-layer"></a>

Taliro uses [Sled](https://github.com/spacejam/sled), an embedded ACID-compliant key-value database.

[//]: # (### 🛷 <ins>Sled Database</ins>)
[//]: # ()
[//]: # (Taliro uses [Sled]&#40;https://github.com/spacejam/sled&#41;, an embedded ACID-compliant key-value database.)
[//]: # ()
[//]: # (**Tree Structure**:)
[//]: # (```)
[//]: # (blockchain_blocks/           # Block data &#40;hash → Block&#41;)
[//]: # (blockchain_heights/         # Height index &#40;u64 → hash&#41;)
[//]: # (blockchain_meta/             # Blockchain metadata &#40;tip pointer&#41;)
[//]: # (utxo/                                # UTXO set &#40;outpoint → UTXO&#41;)
[//]: # (network_identity/             # Node's P2P persistent identity )
[//]: # (outbox_unprocessed/      # Pending outbox entries)
[//]: # (```)
[//]: # ()
[//]: # (**Transaction Support**: Sled provides ACID transactions across multiple trees, enabling atomic operations.)

### 🗃️ <ins>Repository Pattern</ins>

All storage operations flow through repositories.<br />
Domain-level entity repo traits are defined in `domain`.<br />
Application-specific repo traits (if any) would be defined in `application`.<br />
Repository implementations reside in `infrastructure`.

### 🥢 <ins>Unit of Work Pattern</ins>

Taliro defines an abstraction layer over Sled transactions to align with clean architecture principles.<br />
Complex operations rely on units of work for transaction management, ensuring atomicity across repositories within a single database transaction.

---

## <ins>Subsystem Deep Dives</ins> 🤿 <a name="subsystem-deep-dives"></a>

### 🤹🏻‍♀️ <ins>Node (Orchestrator)</ins> <a name="subsystem-deep-dives--node"></a>

**Responsibilities**:
- Central command loop coordinating all subsystem operations
- Sequential command processing from HTTP and P2P sources
- Command routing via dispatcher to appropriate handlers

**Pattern**: Single-threaded event loop with MPSC command queue

**State Machine**: Explicit state transitions (Initialized → Bootstrapped → Started → Running → Terminating)

### ⛓️ <ins>Blockchain</ins> <a name="subsystem-deep-dives--blockchain"></a>

**Responsibilities**:
- Block insertion with continuity validation
- Tip management (active chain head)
- Block retrieval by hash or height

**State Management**:
- **Persistent Storage**: Blocks stored via `BlockchainRepository`
- **Height Index**: Separate tree mapping height to block hash
- **Tip Cache**: In-memory cache (protected by `Mutex`) for fast tip queries
- **Chain Continuity**: Enforces previous hash validation before insertion

**Outbox Pattern**: Block insertions write an outbox entry atomically with the block, ensuring downstream effects (UTXO updates, mempool updates, tip changes) are eventually processed even after crashes.

### 💰 <ins>UTXO Set</ins> <a name="subsystem-deep-dives--utxo-set"></a>

**Responsibilities**:
- Track unspent transaction outputs
- Validate transaction inputs against UTXO set
- Apply block effects (consume inputs, create outputs)

**CQRS Architecture**:
- **UtxoSetReader**: Read-only queries for transaction validation (Query side)
- **UtxoSetWriter**: Write operations during block application (Command side)

**Storage**: Persistent via `UtxoRepository`, keyed by transaction outpoint (hash + output index)

**Block Application**: Atomic transaction deletes consumed UTXOs (inputs) and inserts new UTXOs (outputs)

**Design Benefits**: Separating read and write concerns allows validation to query UTXO state without blocking on write operations

### 🏧 <ins>Mempool</ins> <a name="subsystem-deep-dives--mempool"></a>

**Responsibilities**:
- Transaction queuing for block inclusion
- Conflict detection (duplicate hash prevention)
- Mempool cleanup after block append

**Implementation**: In-memory hash map of pending transactions (protected by `RwLock`)

**Operations**: Add, query, and remove transactions. Cleaned up automatically when blocks are appended.

**Limitations**: No prioritization, size limits, or advanced conflict detection (future enhancements)

### 🌐 <ins>Network</ins> <a name="subsystem-deep-dives--network"></a>

**Responsibilities**:
- P2P peer discovery and connection management
- Message broadcasting via Gossipsub
- Request/response protocol for block sync
- Peer lifecycle tracking

**libp2p Stack**:
- **Transport**: TCP with noise encryption and yamux multiplexing
- **Gossipsub**: Pub/sub for block broadcasting
- **Request/Response**: Custom Taliro protocol for block requests
- **Kademlia**: DHT for peer discovery

**Event Loop**: Runs in separate async task, bidirectional communication with node command loop via MPSC channels

**Peer Store**: Tracks connected peers and their multiaddresses for reconnection

### 🚧 <ins>Block Processing Queue</ins> <a name="subsystem-deep-dives--block-proc-queue"></a>

**Responsibilities**:
- Ensures in-order block processing
- Buffers out-of-order blocks until dependencies arrive
- Prevents concurrent processing of the same block

**Pattern**: Async queue consumer in separate task

**Flow**: Receives blocks → waits for next expected height → validates → applies to blockchain

### 🪡 <ins>Block Sync Queue</ins> <a name="subsystem-deep-dives--block-sync-queue"></a>

**Responsibilities**:
- Manages block-related network fetch operations
- Prevents redundant block requests from peers
- Coordinates synchronization after tip announcements

**Pattern**: Tracks in-progress and completed block requests

**Flow**: Tip announced → calculate missing heights → request blocks → feed to processing queue

### 🛃 <ins>Validation</ins> <a name="subsystem-deep-dives--validation"></a>

**Responsibilities**:
- Single source of truth for validation rules
- Split structural and contextual validation (offline checks vs stateful checks)
- Construct validated types from non-validated inputs

**Strategy**: Fail-fast with detailed error reporting

---

## Network Architecture 🕸️ <a name="network-architecture"></a>

[//]: # (### <ins>P2P Topology</ins>)

Taliro nodes form a mesh network with no central authority.<br />
Peer-to-peer implementation is based on `libp2p`.

Nodes can specify initial peers via configuration.<br />
New peers are discovered via network events or manual addition (dev API).

[//]: # (### Protocol Stack)
[//]: # ()
[//]: # (#### Gossipsub Protocol)
[//]: # (- **Topic**: `"taliro"` &#40;single shared topic&#41;)
[//]: # (- **Messages**: Block announcements &#40;future: transaction propagation&#41;)
[//]: # ()
[//]: # ()
[//]: # (#### Taliro Request/Response Protocol)
[//]: # (- **Purpose**: Block synchronization)
[//]: # (- **Messages**: Block requests by height, block responses)
[//]: # ()
[//]: # (#### Kademlia DHT)
[//]: # (- **Purpose**: Peer discovery and routing)
[//]: # (- **Mode**: Server mode &#40;both provides and queries routing information&#41;)
[//]: # (- **Usage**: Automatic peer discovery in the network mesh)

[//]: # (### <ins>Peer Lifecycle</ins>)

```
Initial Peers → Dial → Connection → Gossipsub Subscribe → Tip Exchange → Sync → Steady State
```

**Reconnection**: Nodes store peer addresses and can reconnect after restarts (unless using ephemeral ports).

---

## <ins>Concurrency Model</ins> 🫛 <a name="concurrency-model"></a>

[//]: # (### <ins>Task Architecture</ins> <a name="concurrency-model--tasks"></a>)

Taliro runs multiple concurrent async tasks:

- **HTTP Server Task**: Handles API requests (`Axum` runtime)
- **Node Event Loop Task**: Sequential command processing
- **Network Event Loop**: `libp2p` swarm event handling
- **Block Processing Task**: `BlockProcessingQueue` consumer
- **Outbox Relay Task**: Polls for unprocessed outbox events

**Note**: Database transactions are held briefly, never spanning async boundaries.

---

## <ins>Transaction Guarantees</ins> ☔ <a name="transaction-guarantees"></a>

### 🔋 <ins>ACID Properties</ins> <a name="transaction-guarantees--acid"></a>

**Atomicity**: State changes across multiple aggregates are committed or rolled back as a single unit, ensuring consistency.

**Consistency**: All validation rules are enforced before state mutations, maintaining blockchain invariants at all times.

**Isolation**: Concurrent operations are serialized to prevent conflicts and ensure deterministic state transitions.

**Durability**: Committed operations are persisted to disk and survive system crashes.

### 🍱 <ins>Outbox Pattern</ins> <a name="transaction-guarantees--outbox"></a>

State changes often produce side effects that must be reliably executed.<br />
However, when these actions span multiple system boundaries, they cannot always be cleanly or reliably handled within a single atomic transaction.<br />
We still need to ensure these side effects are applied, even in the event of crashes.<br />

To address this, Taliro employs the outbox pattern:<br />
An `OutboxEntry` is atomically written alongside the primary state mutation.<br />
A background task continuously polls for unprocessed outbox entries, executes the associated side effects and marks them as processed.<br />
This guarantees at-least-once execution of side effects, though not exactly-once.<br />
Side effect handlers must be idempotent to handle potential duplicates.

```
State Mutation → Outbox Entry Insertion → [Crash?] → Restart → Relay Processes Entry → Completion
```

---

## <ins>Error Handling Strategy</ins> 💀 <a name="error-handling-strategy"></a>

Taliro defines a rich hierarchy of error types to represent various failure modes across the system.<br />
You may inspect the full list of error types under `common/src/error/`.

Errors flow from handlers through command responders to the originating source (HTTP or P2P).<br />
Upon reaching the source, errors are transformed into suitable responses or logged for diagnostics.

Internal logs contain full error details, while public API responses sanitize sensitive information.
