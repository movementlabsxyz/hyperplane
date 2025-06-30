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
├── documentation/       # Project documentation
├── src/
│   ├── bin/            # Binary executables
│   ├── types/          # Core type definitions
│   ├── hyper_ig/       # Hyper Information Gateway
│   ├── hyper_scheduler/# Hyper Scheduler
│   ├── confirmation_layer/ # Confirmation Layer
│   ├── mock_vm/        # Mock Virtual Machine implementation
│   ├── network/        # Network communication
│   └── utils/          # Utility functions and helpers
├── simulator/          # Simulation framework and performance testing
│   ├── src/            # Simulator core logic
│   └── scripts/        # Plotting and analysis scripts
├── submodules/         # Git submodules
│   └── x-chain-vm/     # Virtual Machine implementation
└── tests/              # Integration tests
```

## Development Status

The project is currently in active development. See [PLAN](PLAN.md) for the implementation roadmap and [RULES](RULES.md) for development guidelines.

### Development Setup

This project uses git submodules to manage dependencies that require active development. The main submodule is `x-chain-vm`, which contains the Virtual Machine implementation.

#### Working with Submodules

Clone the repository with submodules:

```bash
git clone --recursive https://github.com/movementlabsxyz/hyperplane.git
```

Or if you've already cloned the repository:

```bash
git submodule update --init --recursive
```

### Current Features

- Basic type definitions and core data structures
- Communication model based on channels between components
- Basic Confirmation Layer implementation that produces blocks per chain
- Basic HyperIG implementation with transaction execution and status management
- Basic HyperScheduler implementation that schedules transactions
- Basic tests per component in their respective module directories (e.g., `src/hyper_ig/tests/`)
- Basic integration tests in `tests/integration`
- Channel-based mock network for testing (no libp2p/gRPC implementation yet)

### Planned Features

- BFT confirmation engine
- Full VM
- Metrics and observability
- Performance profiling
- Production deployment setup
- (optional) libp2p network backend, where necessary

### Running the interactive shell

The project includes an interactive shell for testing and development. To run it:

```bash
cargo run --bin main
```

Logs are enabled by default. The logs are written to `hyperplane.log` in the root directory. You can track the logs in real-time by running in a separate terminal:

```bash
tail -f hyperplane.log
```

### Performance Testing

A simulator tool is available for performance testing. To run it:

```bash
./simulator/run.sh
```

See [simulator/README](simulator/README.md) for details.

### Testing

Run all tests:

```bash
cargo test
```

By default, running `cargo test` will not show logs. To enable logging in tests, you can run:

```bash
HYPERPLANE_LOGGING=true cargo test -- --nocapture
```

We also provide a test runner script:

```bash
#  test_set: 1 for first set of tests, 2 for second set
#  logging:  0 to disable logging, 1 to enable logging
./run_tests.sh <test_set> <logging>
```

## Contributing

Please read [RULES](RULES.md) for development guidelines and contribution rules.
