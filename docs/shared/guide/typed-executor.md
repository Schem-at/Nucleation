# TypedCircuitExecutor Guide

The `TypedCircuitExecutor` provides a high-level API for running redstone circuit simulations with typed inputs and outputs.

## Overview

Instead of manually managing signal strengths and tick counts, the TypedCircuitExecutor lets you work with semantic types like integers, booleans, and ASCII text.

## Quick Start

```rust
use nucleation::{TypedCircuitExecutor, IoType, LayoutFunction, Value};
use std::collections::HashMap;

// Create executor with IO mappings
let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

// Execute with typed inputs
let mut input_values = HashMap::new();
input_values.insert("a".to_string(), Value::Bool(true));
input_values.insert("b".to_string(), Value::Bool(true));

let result = executor.execute(
    input_values,
    ExecutionMode::FixedTicks { ticks: 100 }
)?;

// Get typed output
let output = result.outputs.get("result").unwrap();
assert_eq!(*output, Value::Bool(true));  // AND gate result
```

## IO Type System

The executor supports rich type semantics for circuit inputs/outputs:

### Basic Types

```rust
// Boolean (0 or 15 signal strength)
IoType::Bool

// Unsigned integers (1-32 bits)
IoType::U8
IoType::U16
IoType::U32

// Signed integers
IoType::I8
IoType::I16
IoType::I32

// Floating point
IoType::Float32
IoType::Float64
```

### Complex Types

```rust
// ASCII text (7 bits per character)
IoType::Ascii { length: 16 }

// Fixed-size arrays
IoType::Array {
    element_type: Box::new(IoType::U8),
    length: 10
}

// 2D matrices
IoType::Matrix {
    element_type: Box::new(IoType::Bool),
    rows: 8,
    cols: 8
}

// Structured data
IoType::Struct {
    fields: vec![
        ("x".to_string(), IoType::U16),
        ("y".to_string(), IoType::U16),
    ]
}
```

## Layout Functions

Layout functions define how binary data maps to physical redstone positions:

### OneToOne
One bit per nibble (signal 0 or 15):
```rust
LayoutFunction::OneToOne
```

### Packed4
Four bits per nibble (hex encoding):
```rust
LayoutFunction::Packed4
```

### RowMajor / ColumnMajor
For 2D data like matrices:
```rust
LayoutFunction::RowMajor { width: 8 }
LayoutFunction::ColumnMajor { height: 8 }
```

### Custom
Define your own mapping logic:
```rust
LayoutFunction::Custom {
    spread: Box::new(|bits| { /* bits -> nibbles */ }),
    collect: Box::new(|nibbles| { /* nibbles -> bits */ }),
}
```

## Execution Modes

### Fixed Ticks
Run for a specific number of simulation ticks:
```rust
ExecutionMode::FixedTicks { ticks: 100 }
```

### Until Condition
Run until an output meets a condition (with timeout):
```rust
ExecutionMode::UntilCondition {
    output_name: "ready".to_string(),
    condition: OutputCondition::Equals(Value::Bool(true)),
    timeout_ticks: 1000,
    check_interval: 10,
}
```

### Until Stable
Run until all outputs are stable for N ticks:
```rust
ExecutionMode::UntilStable {
    stable_ticks: 20,
    timeout_ticks: 1000,
}
```

### Until Change
Run until any output changes:
```rust
ExecutionMode::UntilChange {
    timeout_ticks: 1000,
    check_interval: 5,
}
```

## State Management

Control whether simulation state persists between executions:

### Stateless (Default)
Reset to initial state before each execution:
```rust
executor.set_state_mode(StateMode::Stateless);
```

### Stateful
Preserve state between executions:
```rust
executor.set_state_mode(StateMode::Stateful);
```

### Manual
Control reset manually:
```rust
executor.set_state_mode(StateMode::Manual);
executor.reset()?;  // Explicit reset when needed
```

## Building an Executor

### Step 1: Define IO Mappings

```rust
use nucleation::{IoMapping, IoType, LayoutFunction};

let mut inputs = HashMap::new();
inputs.insert("a".to_string(), IoMapping {
    io_type: IoType::Bool,
    layout: LayoutFunction::OneToOne,
    positions: vec![(0, 1, 0)],  // Physical redstone position
});

let mut outputs = HashMap::new();
outputs.insert("result".to_string(), IoMapping {
    io_type: IoType::Bool,
    layout: LayoutFunction::OneToOne,
    positions: vec![(4, 1, 1)],
});
```

### Step 2: Create World with Custom IO

```rust
use nucleation::{MchprsWorld, SimulationOptions};
use mchprs_blocks::BlockPos;

let mut custom_io = Vec::new();
for mapping in inputs.values() {
    for &(x, y, z) in &mapping.positions {
        custom_io.push(BlockPos::new(x, y, z));
    }
}
for mapping in outputs.values() {
    for &(x, y, z) in &mapping.positions {
        custom_io.push(BlockPos::new(x, y, z));
    }
}

let options = SimulationOptions {
    optimize: true,
    io_only: false,
    custom_io,
};

let world = MchprsWorld::with_options(schematic, options)?;
```

### Step 3: Create Executor

```rust
let executor = TypedCircuitExecutor::with_options(
    world,
    inputs,
    outputs,
    options
);
```

## Examples

### AND Gate

```rust
// Create AND gate schematic
let and_gate = create_and_gate_schematic();

// Define IO
let mut inputs = HashMap::new();
inputs.insert("a".to_string(), IoMapping {
    io_type: IoType::Bool,
    layout: LayoutFunction::OneToOne,
    positions: vec![(0, 1, 0)],
});
inputs.insert("b".to_string(), IoMapping {
    io_type: IoType::Bool,
    layout: LayoutFunction::OneToOne,
    positions: vec![(0, 1, 2)],
});

let mut outputs = HashMap::new();
outputs.insert("result".to_string(), IoMapping {
    io_type: IoType::Bool,
    layout: LayoutFunction::OneToOne,
    positions: vec![(4, 1, 1)],
});

// Create executor
let mut executor = create_executor(and_gate, inputs, outputs);

// Test all combinations
let test_cases = vec![
    (false, false, false),
    (false, true, false),
    (true, false, false),
    (true, true, true),
];

for (a, b, expected) in test_cases {
    let mut inputs = HashMap::new();
    inputs.insert("a".to_string(), Value::Bool(a));
    inputs.insert("b".to_string(), Value::Bool(b));
    
    let result = executor.execute(
        inputs,
        ExecutionMode::FixedTicks { ticks: 100 }
    )?;
    
    let output = result.outputs.get("result").unwrap();
    assert_eq!(*output, Value::Bool(expected));
}
```

### 4-Bit Adder

```rust
// Create 4-bit adder schematic
let adder = create_4bit_adder_schematic();

// Define IO (4 bits for each input, 5 bits for output with carry)
let mut inputs = HashMap::new();
inputs.insert("a".to_string(), IoMapping {
    io_type: IoType::U8,  // Using U8 for 4-bit value
    layout: LayoutFunction::OneToOne,
    positions: vec![(0, 1, 0), (0, 1, 1), (0, 1, 2), (0, 1, 3)],
});
inputs.insert("b".to_string(), IoMapping {
    io_type: IoType::U8,
    layout: LayoutFunction::OneToOne,
    positions: vec![(0, 1, 5), (0, 1, 6), (0, 1, 7), (0, 1, 8)],
});

let mut outputs = HashMap::new();
outputs.insert("sum".to_string(), IoMapping {
    io_type: IoType::U8,
    layout: LayoutFunction::OneToOne,
    positions: vec![(40, 1, 0), (40, 1, 1), (40, 1, 2), (40, 1, 3), (40, 1, 4)],
});

// Create executor
let mut executor = create_executor(adder, inputs, outputs);

// Test addition
let mut inputs = HashMap::new();
inputs.insert("a".to_string(), Value::U32(7));  // 0111
inputs.insert("b".to_string(), Value::U32(5));  // 0101

let result = executor.execute(
    inputs,
    ExecutionMode::FixedTicks { ticks: 200 }
)?;

let sum = result.outputs.get("sum").unwrap();
assert_eq!(*sum, Value::U32(12));  // 01100
```

## API Reference

- [Rust API](../api/rust.md#typedcircuitexecutor)
- [JavaScript API](../api/javascript.md#typedcircuitexecutor)
- [Python API](../api/python.md#typedcircuitexecutor)

