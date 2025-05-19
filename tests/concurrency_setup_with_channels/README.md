# Concurrency Setup Test Evolution

This directory contains a series of test implementations to arrive at a working setup with channels.

## Testing Strategy
Each version builds upon the previous one, adding complexity and real-world scenarios:
1. Basic concurrency concepts
2. State management
3. Message processing
4. Block production
5. Chain management
6. Error handling
7. Type safety
8. Full integration

## Test Versions Overview

- **V1**: Simple counter
  - Basic mutex usage
  - Single value updates
  - Simple sleep-based yielding

- **V2**: Complex State Structure
  - Added TestNodeState struct
  - Multiple fields management
  - Message processing simulation

- **V3**: Message Processing
  - Added message channels
  - Basic message queue handling
  - State updates with messages

- **V4**: Block Production
  - Added block structure
  - Block interval timing
  - Message grouping into blocks

- **V5**: Chain-Specific Processing
  - Added chain ID support
  - Per-chain message tracking
  - Chain-specific state management

- **V6**: Subblock Implementation
  - Added subblock structure
  - Chain-specific subblocks
  - Message routing to subblocks

- **V7**: Chain Registration
  - Added chain registration system
  - Chain validation
  - Registration state tracking

- **V8**: Error Handling
  - Added proper error types
  - Chain validation errors
  - Block interval validation

- **V9**: Type-Safe Transactions
  - Added Transaction type
  - Type-safe message handling
  - Structured transaction data

- **V10**: CL Transaction Integration
  - Added CLTransaction type
  - Transaction submission system
  - Chain-specific transaction handling

- **V11**: Real Type Integration
  - Uses actual protocol types
  - Integration with core types
  - Real transaction handling

- **V12**: Node Structure
  - Added node implementations
  - Node communication channels
  - Full node lifecycle

- **V13**: Full Integration
  - Uses actual ConfirmationLayer
  - Real node setup
  - Complete integration
