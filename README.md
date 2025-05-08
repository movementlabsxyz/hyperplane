# Hyperplane

<p align="center">
  <img src="documentation/cat.jpg" alt="A cat silhouette" width="60"/>
  <img src="documentation/box.jpg" alt="A box" width="60"/>
</p>

<p align="center">
  <em>The place where we let the CAT out of the box.</em>
</p>

## Components

### Hyper Scheduler
The coordination layer that manages transaction scheduling and conflict resolution. It maintains the global view of transaction dependencies and ensures consistent ordering.

### Hyper IG (Information Gateway)
Acts as the information gateway for transaction processing, handling the flow of transaction data between components. It manages transaction execution, generates proposals, and ensures proper information routing throughout the system.

### Resolver
Resolves transaction acceptance by combining views from both the scheduler and sequencer. It ensures finality and consistency across the network.

### Confirmation Layer
Provides transaction finality through either centralized or BFT (Byzantine Fault Tolerant) confirmation mechanisms. This layer ensures that transactions are permanently recorded and cannot be reversed.

### Database
Provides state management and transaction storage. Supports both in-memory and file-backed storage options for different deployment scenarios.

### Network
Handles communication between nodes using either mock implementations for testing or real libp2p/gRPC for production deployments.

## Project Structure
```
hyperplane/
├── Cargo.toml
├── hyper_scheduler/         # coordination logic for Crosschain Atomic Transactions
│   └── lib.rs
├── hyper_ig/               # information gateway for tx processing
│   └── lib.rs
├── resolver/               # resolves tx acceptance from coordinator + sequencer view
│   └── lib.rs
├── confirmation/           # simulates centralized or BFT confirmation layer
│   └── lib.rs
├── db/                     # simple in-memory or file-backed state + tx store
│   └── mod.rs
├── network/                # mock or real libp2p/gRPC transport traits
│   └── mod.rs
├── types/                  # shared types: transactions, Crosschain Atomic Transactions, statuses, etc.
│   └── mod.rs
└── bin/
    ├── node.rs             # runs a node with db + confirmation + net
    ├── coordinator.rs      # runs standalone coordinator
    └── simulator.rs        # simulates multi-node exec+resolve flow
```

## Getting Started

[Coming soon]

## Development

[Coming soon]

## License

[Coming soon]

