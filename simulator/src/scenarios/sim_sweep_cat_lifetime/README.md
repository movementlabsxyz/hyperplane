# CAT Lifetime Sweep

Explores how different Cross-Chain Atomic Transaction (CAT) lifetimes affect system performance.

## Key Features

- Sweeps CAT lifetime in blocks with configurable step size
- CAT lifetime determines how long cross-chain transactions remain valid
- Tests impact on success rates and retry patterns
- Explores transaction expiration effects on throughput

## Results

We can see that the number of failed CATs increases substantially if the CAT lifetime is too short. This is particularly pronounced in the below figure when the CAT lifetime is in the range of the delay of one of the chains.

![Failed CATs](./tx_failure_cat.png)

**Figure Parameters:** CAT lifetime sweep (5-20 blocks), block interval=0.02s, TPS=500.0, 2 chains (delay of second chain 5 blocks), 3% CAT ratio, 1000 accounts, 20 runs averaged.
