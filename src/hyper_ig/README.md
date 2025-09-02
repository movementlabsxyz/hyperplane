# Hyper Information Gateway (HIG)

The Hyper Information Gateway is responsible for executing transactions, managing their status, and resolving CAT (Cross-Chain Atomic Transaction) transactions.

## CAT Pending Dependency Configuration

The HIG node now supports a configurable flag that controls whether CAT transactions can depend on pending transactions.

### Background

In the Hyperplane protocol:

- **Normal dependency**: A CAT can depend on another transaction (CAT or regular TX) that has already been committed/executed
- **Pending dependency restriction**: A CAT should NOT be allowed to depend on a key that is currently pending (i.e., a transaction that has been submitted but not yet committed)

### Configuration

The `allow_cat_pending_dependencies` flag controls this behavior:

- **`true`** (default): Allow CATs to depend on pending transactions (current behavior)
- **`false`**: Reject CATs that depend on pending transactions and send a failure message to HS

### Usage

#### Creating a HIG Node with Default Behavior

```rust
use hyperplane::hyper_ig::HyperIGNode;
use tokio::sync::mpsc;

let (receiver_cl_to_hig, _sender_cl_to_hig) = mpsc::channel(100);
let (sender_hig_to_hs, _receiver_hig_to_hs) = mpsc::channel(100);

// Default behavior: allow_cat_pending_dependencies = false
let hig_node = HyperIGNode::new(
    receiver_cl_to_hig,
    sender_hig_to_hs,
    chain_id,
    cat_lifetime
);
```

#### Creating a HIG Node with Restricted Behavior

```rust
use hyperplane::hyper_ig::HyperIGNode;
use tokio::sync::mpsc;

let (receiver_cl_to_hig, _sender_cl_to_hig) = mpsc::channel(100);
let (sender_hig_to_hs, _receiver_hig_to_hs) = mpsc::channel(100);

// Restrict CATs from depending on pending transactions
let hig_node = HyperIGNode::new_with_config(
    receiver_cl_to_hig,
    sender_hig_to_hs,
    chain_id,
    cat_lifetime,
    false // allow_cat_pending_dependencies = false
);
```

#### Runtime Configuration

You can also change the flag at runtime:

```rust
// Get current setting
let current_setting = hig_node.get_allow_cat_pending_dependencies().await;

// Change the setting
hig_node.set_allow_cat_pending_dependencies(false).await;
```

### Behavior Examples

#### With `allow_cat_pending_dependencies = false` (default)

1. CAT A accesses key "account_1" and becomes pending
2. CAT B also accesses key "account_1" 
3. CAT B is allowed to be pending and waits for CAT A to resolve
4. Both CATs can be processed normally

#### With `allow_cat_pending_dependencies = false`

1. CAT A accesses key "account_1" and becomes pending
2. CAT B also accesses key "account_1"
3. CAT B is **rejected** and marked as failed
4. A failure status proposal is sent to the Hyper Scheduler
5. Only CAT A continues processing

### Error Handling

When a CAT is rejected due to pending dependency restrictions, it will:

1. Be marked with `TransactionStatus::Failure`
2. Be removed from the pending transactions set
3. Send a `CATStatusLimited::Failure` status proposal to the Hyper Scheduler
4. Log a detailed message indicating the rejection reason

### Testing

The functionality is thoroughly tested with the following test cases:

- `test_cat_pending_dependency_restriction`: Tests both allowed and restricted behaviors
- `test_cat_pending_dependency_flag_runtime_change`: Tests runtime flag changes

Run the tests with:

```bash
cargo test test_cat_pending_dependency --lib
```

## Transaction Dependency Scenarios

This section summarizes all the different scenarios for how transactions (CATs and regular transactions) handle dependencies and locking.

### Key Concepts

- **Simulation**: Check if transaction would succeed and determine which keys it accesses (no state changes yet)
- **Execution**: Apply the transaction to the final state (actual state changes)
- **Locking**: Reserve keys during pending state to prevent conflicts
- **Dependency Resolution**: Regular or CAT tx gets executed. Notify waiting transactions when dependencies complete

### CAT Transaction Scenarios

#### CAT with No Dependencies (and Success)
1. **Simulate** → Determine it would succeed and identify keys accessed
2. **Lock keys** → Reserve keys during pending state
3. **Set proposed status and send proposal** → Success
4. **Stay pending** → Wait for external resolution via STATUS_UPDATE
5. **When resolved** → Execute, release locks, notify dependents

#### CAT with No Dependencies (and Failure)
1. **Simulate** → Determine it would fail
2. **No locking needed** → Since it will fail anyway
3. **Set proposed status and send proposal** → Failure
4. **Execute immediately** → Apply failure to state (no state changes, gets skipped)
5. **Set final status** → TransactionStatus::Failure

#### CAT with Dependencies (Blocked by Pending Transaction)
1. **Simulate** → Determine success/failure and keys accessed
2. **Check dependencies** → Keys are locked by pending transaction
3. **Fail immediately** → Set proposed status to Failure, send proposal, execute immediately (no state changes)
4. **No locking needed** → Since it fails immediately
5. **No dependency resolution** → Failed CATs don't participate in dependency system

### Regular Transaction Scenarios

#### Regular Transaction with No Dependencies
1. **Simulate** → Determine success/failure and keys accessed
2. **Check dependencies** → Keys are not locked by pending transaction
2. **Execute immediately** → Apply to state, set final status
3. **No locking needed** → Since it executes immediately

#### Regular Transaction with Dependencies (Blocked by Pending Transaction)
1. **Simulate** → Determine success/failure and keys accessed
2. **Check dependencies** → Keys are locked by pending transaction
3. **Become pending** → Wait for dependencies to resolve
4. **Lock keys** → Reserve keys during pending state (like CATs)
5. **When dependencies resolve** → Execute, release locks, notify dependents

### Dependency Resolution Flow

Define 
- **dependency-creator** the transaction that we currently handle is depended on another transaction.
- **dependency-consumer** as a transaction that is depended on the transaction that we currently handle.

When any transaction (CAT or regular) completes:

1. **Release locks** → Remove the lock on the key that is locked by the dependency-creator.
2. **Find dependents** → Look up transactions waiting on the released keys. (dependency-consumers)
3. **Update dependencies** → Remove the dependency-creator from the dependency-consumer's dependency list.
4. **Process dependents** → If all dependencies are resolved of the dependency-consumer, add the dependency-consumer to the indexed list of ordered transactions that are being processed in this block..

### Key Data Structures

- **`key_locked_by_tx`**: Maps keys to the transaction currently locking them
- **`tx_locks_keys`**: Maps transactions to the keys they lock (reverse index)
- **`key_causes_dependencies_for_txs`**: Maps keys to transactions waiting on them
- **`tx_depends_on_txs`**: Maps transactions to their dependencies

### TODO: Regular Transaction Dependencies

**Current Limitation**: Regular transactions cannot depend on other regular transactions

**Issues to Fix**:
- [ ] Missing index for transactions.
- [ ] Regular transactions that execute immediately don't participate in the dependency resolution system
- [ ] The dependency tracking structures are not updated for regular transaction completion

**Implementation Tasks**:
- [ ] Introduce an index for transactions. When a transaction is added to the list of transactions that are being processed in this block, we need to add the transaction at the correct position. (This means it may be handled before all other transactions that are still in that list.)
- [ ] **Lock keys during pending state** (when regular transactions have dependencies)
- [ ] **Release locks and notify dependents** when regular transactions complete
- [ ] **Participate in the dependency resolution system** like CATs do
- [ ] **Modify `handle_regular_transaction`** to lock keys when dependencies exist
- [ ] **Enhance `update_to_final_status_and_update_counter`** to handle regular transaction lock release
- [ ] **Create tests** for regular transaction dependencies
- [ ] **Test mixed scenarios** (regular→CAT, CAT→regular, regular→regular) 