# Morsel Bytecode Specification

## Overview

This document describes the bytecode instruction set, stack model, and function calling mechanism for the Morsel
stack-based virtual machine.

## Table of Contents

1. [Overview](#overview)
2. [Garbage Collector](#garbage-collector)
3. [Stack Model](#stack-model)
4. [Functions and Calling Convention](#functions-and-calling-convention)
5. [Instruction Set](#instruction-set)
6. [Local Variables](#local-variables)
7. [Data Sections and Globals](#data-sections-and-globals)
8. [Labels and Jumps](#labels-and-jumps)
9. [Instruction Encoding](#instruction-encoding)
10. [Executable Format](#executable-format)

## Garbage Collector

The VM uses a stop-the-world (sounds cool, right?), non-concurrent, tracing garbage collector with a bump allocator +
free-list for
reuse. Static allocations are never collected.

### GC triggers

- Allocations increment `allocated_bytes_since_last_gc`.
- GC runs when:
    - A requested allocation doesn't fit into the remaining bump region, or
    - `allocated_bytes_since_last_gc >= gc_threshold` (default is 10% of the heap).
- After GC, `allocated_bytes_since_last_gc` is reset.

## Stack Model

### Stack Semantics

The **stack** is a Last-In-First-Out (LIFO) data structure where all computation occurs. Every instruction either:

- **Pushes** values onto the stack
- **Pops** values from the stack to perform operations
- **Manipulates** the stack structure itself

### Stack Representation

The stack grows upward, with the **top** being the most recently pushed value.

```
Top of Stack
[5]    <- Most recent (top)
[3]
[10]   <- Oldest
Bottom of Stack
```

### Stack Overflow and Underflow

- **Stack overflow** occurs when pushing exceeds available memory.
- **Stack underflow** occurs when popping from an empty stack.

Both conditions halt execution with an error.

## Functions and Calling Convention

### Function Call Sequence

**Argument Order:** Arguments are pushed **left-to-right**. For `add(10, 20)`:

- First argument (10) is pushed first
- Second argument (20) is pushed second
- When the function executes, 10 is deeper in the stack, 20 is on top

**Caller:**

1. Push all arguments onto the stack in order (first argument pushed first).
2. Emit `CALL function_label`.
3. After the call returns, the return value is at the top of the stack.

**Callee:**

1. Arguments are already on the stack in the order they were pushed.
2. Perform function logic.
3. Ensure the return value is at the top of the stack before `RET`.

### Function Declarations

A function declaration in the AST generates:

1. A unique label for the function entry point.
2. Bytecode for the function body.
3. A `RET` instruction at the end.

The label is registered so that `CALL` instructions can reference it.

### Example: Multi-Argument Function

```
Function definition: add(x, y)
  add_label:
  LOAD_LOCAL 0      ; Load x (first argument)
  LOAD_LOCAL 1      ; Load y (second argument)
  ADD               ; Compute x + y
  RET               ; Return result

Function call: result = add(10, 20)
  PUSH_IMM 10       ; Push first argument
  PUSH_IMM 20       ; Push second argument
  CALL add_label    ; Call function
  STORE_LOCAL 0     ; Store result in local variable
```

## Instruction Set

### Stack Manipulation

| Instruction        | Operand | Stack Effect             | Description                            |
|--------------------|---------|--------------------------|----------------------------------------|
| **PUSH_IMM**       | imm     | `[] -> [imm]`            | Push immediate value onto stack        |
| **PUSH_FLOAT_IMM** | imm     | `[] -> [imm]`            | Push f32 immediate value onto stack    |
| **PUSH_HEAP_REF**  | imm     | `[] -> [addr]`           | Push reference to heap value (address) |
| **PUSH_LOCAL_REF** | imm     | `[] -> [index]`          | Push reference to local variable       |
| **POP**            | none    | `[a] -> []`              | Pop and discard top value              |
| **DUP**            | none    | `[a] -> [a, a]`          | Duplicate top value                    |
| **SWAP**           | none    | `[a, b] -> [b, a]`       | Swap top two values                    |
| **ROT**            | none    | `[a, b, c] -> [c, a, b]` | Rotate top three values                |

### Arithmetic

| Instruction | Operand | Stack Effect      | Description                                         |
|-------------|---------|-------------------|-----------------------------------------------------|
| **ADD**     | none    | `[a, b] -> [a+b]` | Pop two values, push sum. Polymorphic (int/string). |
| **SUB**     | none    | `[a, b] -> [a-b]` | Pop two values, push difference                     |
| **MUL**     | none    | `[a, b] -> [a*b]` | Pop two values, push product                        |
| **DIV**     | none    | `[a, b] -> [a/b]` | Pop two values, push quotient (integer division)    |
| **REM**     | none    | `[a, b] -> [a%b]` | Pop two values, push remainder                      |
| **POW**     | none    | `[a, b] -> [a^b]` | Pop two values, push a raised to b                  |
| **NEG**     | none    | `[a] -> [-a]`     | Negate top value                                    |

### Logical & Bitwise

| Instruction | Operand | Stack Effect       | Description             |
|-------------|---------|--------------------|-------------------------|
| **AND**     | none    | `[a, b] -> [a&b]`  | Bitwise AND             |
| **OR**      | none    | `[a, b] -> [a\|b]` | Bitwise OR              |
| **XOR**     | none    | `[a, b] -> [a^^b]` | Bitwise XOR             |
| **NOT**     | none    | `[a] -> [~a]`      | Bitwise NOT             |
| **SLA**     | none    | `[a, b] -> [a<<b]` | Left shift a by b bits  |
| **SRA**     | none    | `[a, b] -> [a>>b]` | Right shift a by b bits |

### Comparison

| Instruction | Operand | Stack Effect           | Description      |
|-------------|---------|------------------------|------------------|
| **EQ**      | none    | `[a, b] -> [a==b?1:0]` | Equal            |
| **NE**      | none    | `[a, b] -> [a!=b?1:0]` | Not equal        |
| **LT**      | none    | `[a, b] -> [a<b?1:0]`  | Less than        |
| **GT**      | none    | `[a, b] -> [a>b?1:0]`  | Greater than     |
| **LE**      | none    | `[a, b] -> [a<=b?1:0]` | Less or equal    |
| **GE**      | none    | `[a, b] -> [a>=b?1:0]` | Greater or equal |

### Memory

| Instruction     | Operand | Stack Effect          | Description                                   |
|-----------------|---------|-----------------------|-----------------------------------------------|
| **LOAD**        | none    | `[addr] -> [value]`   | Pop address, push value at that address       |
| **STORE**       | none    | `[addr, value] -> []` | Pop value and address, store value to address |
| **LOAD_LOCAL**  | index   | `[] -> [value]`       | Load local variable at index                  |
| **STORE_LOCAL** | index   | `[value] -> []`       | Pop value, store to local variable at index   |

### Control Flow

| Instruction | Operand | Stack Effect                       | Description                                    |
|-------------|---------|------------------------------------|------------------------------------------------|
| **JMP**     | label   | `[...] -> [...]`                   | Unconditional jump to label                    |
| **JMPT**    | label   | `[cond] -> [...]`                  | Pop condition, jump if non-zero                |
| **JMPF**    | label   | `[cond] -> [...]`                  | Pop condition, jump if zero                    |
| **CALL**    | label   | `[args...] -> [return_value]`      | Call function, return address saved implicitly |
| **RET**     | none    | `[return_value] -> [return_value]` | Return from function                           |

### Misc

| Instruction | Operand | Stack Effect         | Description            |
|-------------|---------|----------------------|------------------------|
| **NOP**     | none    | `[...] -> [...]`     | No operation           |
| **HALT**    | none    | `[...] -> [...]`     | Stop execution         |
| **SYSCALL** | argc    | `[args...] -> [...]` | Call built-in function |

## Local Variables

### Storage and Indexing

Local variables are stored in a **local frame** associated with each function call. The frame contains slots indexed
from 0 onwards.

**Important:** Function arguments occupy the first slots in the local frame. For example, in `func add(x, y)`:

- `x` is at local index 0
- `y` is at local index 1
- Any additional local variables start at index 2

### Access Instructions

- **LOAD_LOCAL index**: Push the value at local slot `index` onto the stack.
- **STORE_LOCAL index**: Pop a value from the stack and store it in local slot `index`.

### Lifetime

Local variables (including arguments) exist for the duration of a function call. When a function returns via `RET`, the
local frame is destroyed and the next function call gets a fresh frame.

### Limits

A function can have at most **2,147,483,647 local variables**. If you need more you're mentally ill.

### Example: Multiple Locals

```
Function: compute()
  Locals: a (index 0), b (index 1), c (index 2), d (index 3)
  
  Bytecode:
    PUSH_IMM 10       ; Push 10
    STORE_LOCAL 0     ; a = 10
    
    PUSH_IMM 20       ; Push 20
    STORE_LOCAL 1     ; b = 20
    
    LOAD_LOCAL 0      ; Push a
    LOAD_LOCAL 1      ; Push b
    ADD               ; Compute a + b
    STORE_LOCAL 2     ; c = a + b
    
    LOAD_LOCAL 2      ; Push c
    PUSH_IMM 2        ; Push 2
    MUL               ; Compute c * 2
    STORE_LOCAL 3     ; d = c * 2
    
    LOAD_LOCAL 3      ; Push d (return value)
    RET               ; Return d
```

## Data Sections and Globals

### Global Data Storage

Global variables, string literals, and array constants are stored in a **data blob** (managed by the `Executable`). It's
a fixed, read-only section of memory allocated at executable load time.

### How It Works

1. **Allocation**: During code generation, call `Executable::insert_data(id, bytes, name)` to store data.
2. **Reference**: Use `PUSH_HEAP_REF data_id` to push the data section ID onto the stack.
3. **Resolution**: The linker resolves the data ID to a memory address via the `RelocationTable`.
4. **Access**: Use `LOAD` or `STORE` to read from or write to the resolved address.

## Labels and Jumps

### Label Resolution

Labels are resolved during **executable construction** by the linker. Each label maps to an instruction offset (the
position of that instruction in the bytecode).

When you emit a `CALL`, `JMP`, `JMPT`, or `JMPF` instruction, you reference the label by its **ID**. The linker later
resolves this ID to the actual instruction offset.

### Example: Conditional Jump

```
Condition: x > 5

Bytecode:
  LOAD_LOCAL 0    ; Push x
  PUSH_IMM 5      ; Push 5
  GT              ; Pop both, push (x > 5) as 1 or 0
  JMPF else_label ; If false (0), jump to else

  ; Then branch
  PUSH_IMM 10
  JMP end_label

  ; Else branch
  else_label:
  PUSH_IMM 20

  end_label:
  ; Continue (value is on stack)
```

## Instruction Encoding

### Format

Instructions that doesn't have operand encoded as `[Opcode (u8)]`
Instructions that need operand encoded as `[Opcode (u8)][Operand (i32)]`

## Executable Format

The `Executable` struct is the compiled output of the Morsel compiler. It contains:

| Component        | Description                                             |
|------------------|---------------------------------------------------------|
| **Header**       | Magic number, version info, offsets, entry point offset |
| **Instructions** | Array of bytecode instructions                          |
| **Data Blob**    | Raw bytes for global data (strings, arrays, constants)  |

### Loading and Execution

1. The VM reads the header to validate the executable.
2. Instructions are loaded into instruction memory.
3. The data blob is loaded into heap memory.
4. Execution begins at the `main` function.
