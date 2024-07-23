// Check for a Supermajority.
//
// Write a circuit which checks whether 2/3 or more ballots approved the motion (supermajority).
// Each ballot is a number, with 0 meaning 'no' and any other number meaning 'yes'.
// The circuit should return 1 to indicate the motion passes and 0 to indicate it fails.
//
// For example:
//  //! test [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] => [0]
//  //! test [1, 1, 1, 1, 1, 1, 1, 1, 1, 1] => [1]
//  //! test [0, 7, 8, 0, 1, 1, 0, 0, 2, 2] => [0]
//  //! test [0, 7, 8, 0, 1, 1, 0, 3, 2, 2] => [1]
//
// The format above is also used to check circuits with `cargo test`. Simply move them to their own
// line, similar to test annotations in `loopAdd.ts` and `greaterThan10.ts`.

export default function main(
  // The main function has this quirky signature to tell CircuitScript to instantiate it for exactly
  // 10 inputs. However we can implement the general algorithm by writing `impl` which simply takes
  // an array of inputs with unspecified length.
  ballot0: number,
  ballot1: number,
  ballot2: number,
  ballot3: number,
  ballot4: number,
  ballot5: number,
  ballot6: number,
  ballot7: number,
  ballot8: number,
  ballot9: number,
) {
  return impl([
    ballot0,
    ballot1,
    ballot2,
    ballot3,
    ballot4,
    ballot5,
    ballot6,
    ballot7,
    ballot8,
    ballot9,
  ]);
}

function impl(ballots: number[]) {
  // TODO: Return true iff 2/3 or more ballots are non-zero. (In the resulting circuit, true will
  // be converted to 1 and false will be converted to 0.)
}
