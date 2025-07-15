# CAT Pending Dependencies Sweep

Explores how the ALLOW_CAT_PENDING_DEPENDENCIES flag affects system performance.
It tests exactly two values to understand the impact of CAT transaction restrictions on locked keys:

- false: CATs are rejected when they depend on locked keys
- true: CATs are allowed to depend on locked keys (current behavior)

## Key Features

- Tests exactly 2 values (false/true) for the flag
- Controls whether CAT transactions can depend on locked keys

