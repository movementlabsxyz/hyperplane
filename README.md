# Hyperplane

<p align="center">
  <img src="documentation/cat.jpg" alt="A cat silhouette" width="60"/>
  <img src="documentation/box.jpg" alt="A box" width="60"/>
</p>

<p align="center">
  <em>The place where we let the CAT out of the box.</em>
</p>

## Cross-chain Atomic Transactions (CATs)

Hyperplane is a protocol for coordinating and executing cross-chain atomic transactions (CATs).

Cross-chain atomic transactions are atomic transactions that span multiple chains. They must only be executed and applied to the persistent state of the system if all participating chains simulated the transaction successfully.

While CATs are pending resolution, their state changes (and the state changes of dependent transactions) are stored in temporary forks (or *superpositions*). Once the CAT is resolved through the Hyperplane protocol, the state of the system is finalized.

## Components

### Hyper Scheduler (HS)
The coordination layer that manages transaction scheduling and conflict resolution. It maintains the global view of transaction dependencies and ensures consistent ordering.

### Hyper Information Gateway (HIG)
Acts as the information gateway for transaction processing, handling the flow of transaction data between components. It manages transaction execution, generates proposals, and ensures proper information routing throughout the system.

### Resolver
Resolves transaction acceptance by combining views from both the scheduler and sequencer. It ensures finality and consistency across the network.

### Confirmation Layer (CL)
Provides transaction finality through either centralized or BFT (Byzantine Fault Tolerant) confirmation mechanisms. This layer ensures that transactions are permanently recorded and cannot be reversed.

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
├── resolver/               # resolves tx acceptance from hyper_scheduler + sequencer view
│   └── lib.rs
├── confirmation/           # simulates centralized or BFT confirmation layer
│   └── lib.rs
├── network/                # mock or real libp2p/gRPC transport traits
│   └── mod.rs
├── types/                  # shared types: transactions, Crosschain Atomic Transactions, statuses, etc.
│   └── mod.rs
└── bin/
    ├── node.rs             # runs a node with confirmation + net
    ├── hyper_scheduler.rs  # runs standalone hyper scheduler
    └── simulator.rs        # simulates multi-node exec+resolve flow
```

## Testing

### Running Tests

To run all tests:
```bash
cargo test
```

To run integration tests:
```bash
cargo test --test integration_test
```

To run a specific integration test:
```bash
cargo test --test integration_test test_confirmation_node_basic
```

To run tests with debug output:
```bash
cargo test -- --nocapture --test-threads=1
```

### Test Structure

- Unit tests are located in each module's `#[cfg(test)]` section
- Integration tests are in `tests/integration_test.rs`
- As we develop new components, we add corresponding integration tests

## Getting Started

[Coming soon]

## Development

[Coming soon]

## License

[Coming soon]