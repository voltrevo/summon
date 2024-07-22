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

## Limitations

- You can't use a signal as an array index
