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

// Default behavior: allow_cat_pending_dependencies = true
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

#### With `allow_cat_pending_dependencies = true` (default)

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