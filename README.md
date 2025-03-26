# Mantle Transaction Conflict Analysis

This project analyzes Mantle blockchain transactions to determine the conflict rate between transactions within blocks. The conflict rate helps us understand what percentage of transactions could potentially be parallelized.

## Overview

The analysis identifies two types of conflicts:

1. Multiple MNT transfers from the same source address
2. Multiple storage slot accesses (reads/writes) to the same contract storage slot

## Requirements

- Rust 1.70+
- Access to a Mantle RPC endpoint

## Setup

1. Create a `.env` file in the project root with your Mantle RPC URL:

```
MANTLE_URL=your_rpc_url_here
```
note: The `debug_getRawTransaction` method is must be enabled in the RPC endpoint.

2. Install dependencies:

```bash
cargo build --release
```

## Usage

Run the analysis:

```bash
cargo run --release
```

The program will:

- Analyze specified blocks (currently set to analyze 2 blocks starting from block 77075444)
- Process transactions sequentially
- Output progress updates and final statistics including:
  - Total blocks analyzed
  - Invalid blocks encountered
  - Total transactions analyzed
  - Number of dependent transactions
  - Overall dependency ratio
  - Conflict counts by type (same-source and storage-slot conflicts)

## Note

The analysis focuses on two main types of conflicts:
1. Multiple MNT transfers from the same source address
2. Storage slot conflicts where multiple transactions read/write to the same contract storage slot

This helps identify potential transaction dependencies that could affect parallel execution.