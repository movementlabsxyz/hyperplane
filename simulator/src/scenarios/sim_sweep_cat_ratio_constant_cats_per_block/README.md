# CAT Ratio with Constant CATs per Block Sweep

This sweep explores how different target TPB values affect system performance while maintaining a constant number of CATs per block.

## Concept

The CAT ratio is calculated as:
```
cat_ratio = constant_cats_per_block / target_tpb
```

This ensures that regardless of target TPB, the same number of CATs are generated per block, allowing us to isolate the effect of transaction rate from the effect of CAT frequency.
