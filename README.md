# CircuitScript

*Write circuits in TypeScript*

Based on [ValueScript](https://github.com/voltrevo/ValueScript).

## Setup

```sh
cargo build
export PATH="$PATH:$PWD/target/debug"
```

## Usage

```sh
csc main.ts
```

This will generate the circuit in [bristol format](https://nigelsmart.github.io/MPC-Circuits/) at `output/circuit.txt` and a description of the inputs, outputs, and constants at `output/circuit_info.json`.

## Example

```ts
// examples/loopAdd.ts

const iterations = 3;

export default function main(input: number) {
  let res = 0;

  for (let i = 0; i < iterations; i++) {
    res += input;
  }

  return res;
}
```

```sh
csc examples/loopAdd.ts
```

```
# output/circuit.txt

2 3
1 1
1 1

2 1 0 0 1 AAdd
2 1 1 0 2 AAdd
```

```jsonc
// output/circuit_info.json

{
  "input_name_to_wire_index": {
    "input": 0
  },
  "constants": {},
  "output_name_to_wire_index": {
    "main": 2
  }
}
```

## Signal-Dependent Branching

Building a circuit from a program with a fixed path is relatively straightforward. The real power
of CircuitScript is its ability to handle signal-dependent branches - where the program follows a
different path depending on the input. For example:

```ts
// examples/greaterThan10.ts

export default function main(x: number) {
  if (x > 10) {
    return 10;
  }

  return 0;
}
```

```
2 1 0 1 2 AGt
2 1 2 1 3 AMul
```

Above, the constant 10 is used for wire 1, so the circuit is `output = (x > 10) * 10`.

CircuitScript can also handle more complex branching, so you can use loops and even things like
`continue`, `break`, and `switch`. You can also conditionally throw exceptions as long as you
catch them.

To achieve this, CircuitScript has a general solution to handle any conditional jump instruction.
A conditional jump generates a new evaluation branch, and each branch tracks a multiplier signal.
CircuitScript dynamically manages these branches and merges them when they reach the same location.

However, it is easy to write programs which branch indefinitely and never consolidate into a single
fixed circuit. Programs like this become infinite loops:

```ts
for (let i = 0; i < input; i++) {
  sum += i;
}
```

A traditional runtime can terminate shortly after `i` reaches `input`, but because `input` isn't
known during compilation, CircuitScript will get stuck in a loop as it adds more and more circuitry
to handle larger and larger values of `input` forever.

## Limitations

- You can't use a signal as an array index

## Exercises

If you'd like to try your hand at CircuitScript but you're not sure where to start, I have prepared
some exercises you might find interesting:
- [Check Supermajority](./examples/exercises/checkSuperMajority.ts)
- [Approval Voting](./examples/exercises/approvalVoting.ts)
- [Sneaky Tic-Tac-Toe](./examples/exercises/sneakyTicTacToe.ts)
- [Asset Swap](./examples/exercises/assetSwap.ts)
- [Poker Hands](./examples/exercises/pokerHands.ts)
