# Hyperplane Implementation Plan

## Project Structure
- [x] Initial `Cargo.toml` setup with basic dependencies
- [x] Directory structure creation
- [x] Main `lib.rs` with module declarations

## Integration Testing
- [x] Basic integration test setup
- [x] Confirmation node integration tests
- [ ] Execution node integration tests
- [ ] Hyper Scheduler integration tests
- [ ] Hyper IG integration tests
- [ ] End-to-end protocol tests

## Core Modules

### Types Module (`src/types/mod.rs`)
- [x] Basic type definitions
  - [x] `TransactionId`
  - [x] `Transaction`
  - [x] `TransactionStatus`
  - [x] `CAT`
- [ ] Additional required types:
  - [ ] `NodeId`, `PeerId`, `ChainId`
  - [ ] Network message types
  - [ ] `ExecutionProposal`, `CrosschainAtomicTransactionStatus`, `CrosschainAtomicTransactionId`
  - [ ] State model types (e.g. `LedgerState`, `ChangeSet`)

### Hyper-Scheduler Module (`src/hyper_scheduler/mod.rs`)
- [x] Basic trait definition
- [ ] Crosschain Atomic Transaction resolution coordination logic
- [ ] Proposal integration
- [ ] Tests
- [ ] Documentation

### Hyper-IG Module (`src/hyper_ig/mod.rs`)
- [x] Basic trait definition
- [ ] Execution logic
- [ ] Transaction simulation
- [ ] Proposal generation
- [ ] Tests
- [ ] Documentation

### Resolver Module (`src/resolver/mod.rs`)
- [x] Basic trait definition
- [ ] Resolution of accepted/postponed sets
- [ ] Integration with Hyper-Scheduler and Sequencer
- [ ] Tests
- [ ] Documentation

### Confirmation Layer Module (`src/confirmation/mod.rs`)
- [x] Basic trait definition
- [x] Simple node implementation
- [ ] Centralized confirmation implementation
- [ ] BFT confirmation implementation (`src/confirmation/bft.rs`)
- [ ] Tests
- [ ] Documentation

### State Module (`src/state/mod.rs`)
- [ ] Basic trait definition
- [ ] In-memory state engine
- [ ] File-backed ledger storage (optional)
- [ ] Tests
- [ ] Documentation

### Network Module (`src/network/mod.rs`)
- [ ] Basic trait definition
- [ ] Mock transport
- [ ] `libp2p` backend
- [ ] gRPC implementation (optional)
- [ ] Tests
- [ ] Documentation

## Binary Targets

### Node Binary (`src/bin/node.rs`)
- [ ] Node initialization
- [ ] CLI + Config
- [ ] Component orchestration (state + net + resolver + confirmation)
- [ ] Tests

### Hyper-Scheduler Binary (`src/bin/hyper_scheduler.rs`)
- [ ] Standalone Hyper-Scheduler setup
- [ ] CLI interface
- [ ] Config loading
- [ ] Tests

### Simulator Binary (`src/bin/simulator.rs`)
- [ ] Multi-node orchestration
- [ ] Configurable scenario setup
- [ ] Execution + coordination flow simulation
- [ ] Dependency graph output (e.g. dot export)
- [ ] Tests

## Documentation
- [ ] Per-module docs
- [ ] Public API documentation
- [ ] Example usage (scenarios, CLI)
- [ ] Architecture diagrams
- [ ] Setup guide
- [ ] Contribution guide

## Testing
- [ ] Unit tests for all core components
- [ ] Integration tests for Crosschain Atomic Transaction protocol execution
- [ ] End-to-end test: tx submission → resolution
- [ ] Fuzzing or `proptest` for state edge cases
- [ ] CI/CD (basic GitHub Actions)

## Future Enhancements
- [ ] BFT confirmation engine
- [ ] Metrics + observability
- [ ] Real transport: `libp2p`, gRPC
- [ ] Performance profiling
- [ ] Production deployment setup
- [ ] Timeout config tuning & adversarial scenarios
