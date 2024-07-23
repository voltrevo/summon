// Sneaky Tic-Tac-Toe.
//
// Let's add some hidden information to tic-tac-toe to make it more interesting and suitable for
// MPC. It's the same game, but each player also gets a hidden move.
//
// Each player chooses their hidden move at the start of the game and computes:
//  commitment = hash(salt, movePos)
//
// All other moves are public. If a player attempts to play an existing move, including any of the
// hidden moves, they immediately lose the game.
//
// Otherwise, the winner is decided by the usual tic-tac-toe rules.
//  https://en.wikipedia.org/wiki/Tic-tac-toe
//
// This circuit allows players to compute the correct outcome for a single move of the game. The
// circuit should check that hash(salt, hiddenPos) of each player is equal to their commitment,
// and failure to do so should give victory to the opponent.
//
// Valid moves are 0 for top-left to 8 for bottom-right (see grid layout). An invalid move gives
// victory to the opponent.
//
// Outputs:
//   0 to indicate that play should continue (a valid move was played and nothing happened yet)
//   1 to indicate that player 1 has won
//   2 to indicate that player 2 has won
//  13 to indicate that the hidden moves were equal (which is a null result)
//  14 to indicate anything else was invalid / inconsistent
//     (generally though, the circuit behavior for invalid public inputs is not important - these
//     should be handled outside MPC)

import hash from '../lib/hash.ts';

export default function main(
  // shared public inputs
  grid0: number, // layout:
  grid1: number, // grid0 grid1 grid2
  grid2: number, // grid3 grid4 grid5
  grid3: number, // grid6 grid7 grid8
  grid4: number,
  grid5: number, // for gridN:
  grid6: number, //  0 means empty
  grid7: number, //  1 means player 1 has played there
  grid8: number, //  2 means player 2 has played there
  player1Commitment: number, // should equal hash(player1Salt, player1HiddenPos)
  player2Commitment: number, // should equal hash(player2Salt, player2HiddenPos)
  currentPlayer: number, // 1 for player 1's turn, 2 for player 2's turn

  // player 1
  player1Salt: number,
  player1HiddenPos: number, // 0 for top-left, 8 for bottom-right (see grid layout)

  // player 2
  player2Salt: number,
  player2HiddenPos: number, // 0 for top-left, 8 for bottom-right (see grid layout)

  // current player
  movePos: number, // 0 for top-left, 8 for bottom-right (see grid layout)
) {
  // check the commitments are correct
  // check the grid for all win conditions
}
