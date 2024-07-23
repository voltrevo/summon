// Poker Hands.
//
// Write a circuit to compare two poker hands.
//
// Using this circuit directly you could simply claim you have a royal flush, but in practice you
// would incorporate this into a larger circuit.
//
// Poker hands and their rankings are described here:
//  https://en.wikipedia.org/wiki/List_of_poker_hands
//
// Cards are encoded like this:
//   0-12: Hearts Ace to King
//  13-25: Clubs Ace to King
//  26-38: Diamonds Ace to King
//  39-51: Spades Ace to King
//
// Output:
//  0: hands equal
//  1: player1's hand wins
//  2: player2's hand wins

export default function main(
  player1Card1: number,
  player1Card2: number,
  player1Card3: number,
  player1Card4: number,
  player1Card5: number,

  player2Card1: number,
  player2Card2: number,
  player2Card3: number,
  player2Card4: number,
  player2Card5: number,
) {
  // TODO
}
