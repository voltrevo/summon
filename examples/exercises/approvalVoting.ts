// Approval Voting.
//
// About: https://www.youtube.com/watch?v=orybDrUj4vA
//
// A simple voting system allowing each participant to vote based on whether they approve of each
// option. Voters are allowed to approve of multiple options. The result is the option with the
// most approvals.
//
// Example:
//  //! test [1, 1, 0,   1, 1, 0,   0, 1, 0,   0, 1, 1,   0, 1, 1,   0, 1, 1] => [1]
//
// Output meanings:
//   0: Steak Shack
//   1: Burger Barn
//   2: Veggie Villa

export default function main(
  steakShack0: number, burgerBarn0: number, veggieVilla0: number,
  steakShack1: number, burgerBarn1: number, veggieVilla1: number,
  steakShack2: number, burgerBarn2: number, veggieVilla2: number,
  steakShack3: number, burgerBarn3: number, veggieVilla3: number,
  steakShack4: number, burgerBarn4: number, veggieVilla4: number,
  steakShack5: number, burgerBarn5: number, veggieVilla5: number,
) {
  return impl([
    {
      steakShack: steakShack0 !== 0,
      burgerBarn: burgerBarn0 !== 0,
      veggieVilla: veggieVilla0 !== 0,
    },
    {
      steakShack: steakShack1 !== 0,
      burgerBarn: burgerBarn1 !== 0,
      veggieVilla: veggieVilla1 !== 0,
    },
    {
      steakShack: steakShack2 !== 0,
      burgerBarn: burgerBarn2 !== 0,
      veggieVilla: veggieVilla2 !== 0,
    },
    {
      steakShack: steakShack3 !== 0,
      burgerBarn: burgerBarn3 !== 0,
      veggieVilla: veggieVilla3 !== 0,
    },
    {
      steakShack: steakShack4 !== 0,
      burgerBarn: burgerBarn4 !== 0,
      veggieVilla: veggieVilla4 !== 0,
    },
    {
      steakShack: steakShack5 !== 0,
      burgerBarn: burgerBarn5 !== 0,
      veggieVilla: veggieVilla5 !== 0,
    },
  ]);
}

type Ballot = {
  steakShack: boolean;
  burgerBarn: boolean;
  veggieVilla: boolean;
};

function impl(ballots: Ballot[]) {
  // TODO
}
