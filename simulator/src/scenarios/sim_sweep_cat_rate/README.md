# CAT Rate Sweep

Varies the ratio of CAT transactions from 0.0 (no CATs) to maximum values.

## Key Features

- Sweeps CAT ratio from 0.0 to configurable maximum
- Configurable step size for ratio increments

## Results

Despite increased CAT ratio, the number of pending regular transactions is not increased much, indicating that the system maintains stable performance for regular transactions even with higher CAT loads.

![Pending Regular Transactions](./tx_pending_regular.png)

**Figure Parameters:** CAT ratio sweep (0.0-0.15), block interval=0.05s, TPS=100.0, 2 chains (delay of second chain 5 blocks), CAT lifetime=10 blocks, 1000 accounts, 20 runs averaged.
