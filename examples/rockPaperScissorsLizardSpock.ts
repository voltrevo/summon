const ROCK = 1;
const PAPER = 2;
const SCISSORS = 3;
const LIZARD = 4;
const SPOCK = 5;

const winConditions = [
  [SCISSORS, /* cuts */ PAPER],
  [PAPER, /* covers */ ROCK],
  [ROCK, /* crushes */ LIZARD],
  [LIZARD, /* poisons */ SPOCK],
  [SPOCK, /* smashes */ SCISSORS],
  [SCISSORS, /* decapitates */ LIZARD],
  [LIZARD, /* eats */ PAPER],
  [PAPER, /* disproves */ SPOCK],
  [SPOCK, /* vaporizes */ ROCK],
  [ROCK, /* crushes */ SCISSORS],
];

export default function main(player1: number, player2: number) {
  let res = 0;

  for (const [winningChoice, losingChoice] of winConditions) {
    res += +(player1 === winningChoice && player2 === losingChoice);
    res += 2 * +(player2 === winningChoice && player1 === losingChoice);
  }

  return res;
}
