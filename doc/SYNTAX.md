# Morsel Syntax Guide

Guide to Morsel syntax and language features. This document covers the core language constructs you need
to write morsel programs.

## Table of Contents

1. [Comments](#comments)
2. [Variables and Mutability](#variables-and-mutability)
3. [Data Types](#data-types)
4. [Operators](#operators)
5. [Control Flow](#control-flow)
6. [Loops](#loops)
7. [Functions](#functions)

## Comments

Comments explain your code without affecting execution.

**Single-line comments:**

```morsel
// This is a single-line comment
let x = 5; // Comment after code
```

## Variables and Mutability

Variables store data values. By default, variables are **immutable** in morsel.

### Immutable Variables

```morsel
let x = 5;
// x = 6; // Error: cannot assign to immutable variable
```

### Mutable Variables

Use the `mut` keyword to make a variable mutable:

```morsel
let mut y = 5;
y = 6; // OK
```

### Shadowing

You can declare a new variable with the same name, shadowing the previous one:

```morsel
let x = 5;
let x = x + 1; // x is now 6
let x = x * 2; // x is now 12
```

## Data Types

Morsel is **statically** and **strictly** typed! The type of variable is known at compile time and cannot change,
preventing type errors during program execution.

### Built-in Types

- **Integer:** `int` - 32-bit integer
- **Float:** `float` - 32-bit floating-point number
- **String:** `string` - Text data
- **Boolean:** `bool` - Boolean value (true/false)

### Arrays

Morsel supports both **fixed-size** and **dynamic arrays**.

**Fixed-size arrays** have their length known at compile-time:

```morsel
let mut x: [int: 3] = [0, 1, 2];
x[0] = 1; // OK
// x[10] = 5; // Error: compile-time bounds check
```

Fixed arrays are **safe** because out-of-bounds access is caught at compile-time.

**Dynamic arrays** have their length determined at runtime:

```morsel
let mut x: [int] = [0, 1, 2];
x[0] = 1; // OK
x[10] = 5; // Compiles, but may fail at runtime
```

**Warning:** Dynamic arrays don't have compile-time bounds checking. Out-of-bounds access compiles but crashes at
runtime. Use fixed arrays when you know the size.

## Type System

### Type Inference

Morsel automatically infers types from context:

```morsel
let x = 5;                // inferred as int
let y = 3.14;             // inferred as float
let name = "hi";          // inferred as string
let input = get_string(); // inferred as string from func return
```

### Explicit Type Annotations

When inference isn't clear, provide explicit types:

```morsel
let x: int = 5;
let items: [string: 3] = ["a", "b", "c"];
```

## Operators

### Arithmetic Operators

```morsel
let a = 10;
let b = 3;

let sum = a + b; // 13
let diff = a - b; // 7
let prod = a * b; // 30
let quot = a / b; // 3 (integer division)
let rem = a % b; // 1 (remainder)
```

### Comparison Operators

```morsel
let a = 5;
let b = 3;

a == b; // false
a != b; // true
a > b;  // true
a < b;  // false
a >= b; // true
a <= b; // false
```

### Logical Operators

```morsel
let t = true;
let f = false;

t && f; // false (AND)
t || f; // true (OR)
!t;     // false (NOT)
```

## Control Flow

In Morsel, `if/else` is an **expression**, meaning it returns a value:

```morsel
let x = 5;

let message = if (x > 0) {
    "positive"
} else {
    "non-positive"
}

println(message);
```

You can also use it as a statement:

```morsel
if (x > 0) {
    println("x is positive");
} else {
    println("x is not positive");
}
```

## Loops

### while Loops

```morsel
let mut i = 0;
while (i < 5) {
    println(i);
    i += 1;
}
```

### for Loops

```morsel
for (let mut i = 0; i < 5; i += 1) {
    println(i);
}
```

### Loop Control

```morsel
// break - Exit the loop
while (true) {
    if (condition) {
        break;
    }
}

// continue - Skip to next iteration
for (let mut i = 0; i < 10; i += 1) {
    if (i % 2 == 0) {
        continue;
    }
    println(i);
}
```

## Functions

Functions are declared with the `func` keyword.

### Basic Function

```morsel
func add(a: int, b: int): int {
	a + b // Implicit return
}

let result = add(5, 3); // 8
```

### Function with No Return Value

```morsel
func print_number(x: int) {
	println("The number is: ", x);
}
```

### Early Return

```morsel
func check_age(age: int): string {
	if (age < 18) {
		return "Too young";
	}
	"Old enough"
}
```