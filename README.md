# Memento

A tiny stack-based bytecode VM you can run forward and backward.

Memento is a small stack-based virtual machine written in Rust. Give it a text
file with simple maths instructions, then run it normally, watch a trace, or
move through it one instruction at a time.

It is intentionally small. The point is to make bytecode execution and
time-travel debugging easy to understand.

## Try it

You need the Rust toolchain.

```
cargo run -- examples/math.mem
# result: 5
```

## The program format

Memento reads one instruction per line. Empty lines and everything after `#`
are ignored.

| Instruction     | What it does                                     |
| --------------- | ------------------------------------------------ |
| `push <number>` | Puts a number on the stack.                      |
| `add`           | Removes two numbers and pushes their sum.        |
| `subtract`      | Removes two numbers and pushes left minus right. |
| `multiply`      | Removes two numbers and pushes their product.    |
| `halt`          | Stops the program.                               |

Example (`examples/math.mem`) calculates `(2 + 3) * 4 - 15`:

```
push 2
push 3
add
push 4
multiply
push 15
subtract
halt
```

Arithmetic problems — too few numbers on the stack, or a result that is too
big for the machine's numbers — print an error and stop the program.

## Run modes

**Normal mode** — runs the program and prints the final result.

```
cargo run -- examples/math.mem
```

**Trace mode** — shows the next instruction and the stack before and after it.

```
cargo run -- examples/math.mem --trace
```

**Debug mode** — run a single instruction at a time.

```
cargo run -- examples/math.mem --debug
```

| Key | What it does                                                               |
| --- | -------------------------------------------------------------------------- |
| `n` | Run the next instruction.                                                  |
| `b` | Restore the stack and instruction pointer from before the last instruction.|
| `q` | Quit the debugger.                                                         |

## Test it

```
cargo test
```

## How reverse stepping works

Before Memento runs an instruction, it saves a small snapshot containing the
stack and instruction pointer. Pressing `b` restores the most recent snapshot.
It is simple and not memory-efficient yet, but it makes the idea easy to see.

## Project layout

- `src/parser.rs` — reads a `.mem` file and turns each line into an instruction.
- `src/vm.rs` — the stack, the instruction pointer, and the snapshot history.
- `src/main.rs` — the command-line interface and the three run modes.
- `examples/` — sample `.mem` programs.
