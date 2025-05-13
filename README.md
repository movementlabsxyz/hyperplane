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

### Hyper Information Gateway (HIG)
The Hyper IG is responsible for:
- Executing transactions and managing their status
- Generating proposals for CAT transactions
- Resolving CAT transactions based on hyper_scheduler and sequencer views
- Managing transaction dependencies and state changes

### Hyper Scheduler (HS)
The coordination layer that:
- Manages transaction scheduling and conflict resolution
- Maintains the global view of transaction dependencies
- Ensures consistent ordering of transactions
- Coordinates CAT resolution across chains

### Confirmation Layer (CL)
Provides transaction finality through:
- Centralized confirmation mechanism
- BFT (Byzantine Fault Tolerant) confirmation mechanism (planned)
- Ensures transactions are permanently recorded and cannot be reversed
- Manages chain registration and block production

### Network
Handles communication between nodes using:
- Mock implementations for testing
- libp2p backend (planned)
- gRPC implementation (optional)

## Project Structure
```
hyperplane/
├── src/
│   ├── types/           # Core type definitions
│   ├── hyper_ig/        # Hyper Information Gateway
│   ├── hyper_scheduler/ # Hyper Scheduler
│   ├── confirmation/    # Confirmation Layer
│   └── network/         # Network communication
├── tests/               # Integration tests
├── examples/            # Example usage
└── documentation/       # Project documentation
```

## Development Status

The project is currently in active development. See [PLAN.md](PLAN.md) for the implementation roadmap and [RULES.md](RULES.md) for development guidelines.

### Current Features
- Basic type definitions and core data structures
- Hyper IG implementation with transaction execution and status management
- Confirmation layer with centralized confirmation mechanism
- Basic integration tests for core components

### Planned Features
- BFT confirmation engine
- libp2p network backend
- Metrics and observability
- Performance profiling
- Production deployment setup

## Testing

```
cargo test  mytest -- --test-threads=1 --nocapture
```

## Contributing

Please read [RULES.md](RULES.md) for development guidelines and contribution rules.

## License

This project is licensed under the MIT License - see the LICENSE file for details.