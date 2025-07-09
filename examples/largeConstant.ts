//! test [1000000000000] => [false]
//! test [1000000000001] => [true]

const oneTrillion = 1_000_000_000_000;

export default (io: Summon.IO) => {
  const a = io.input('alice', 'a', summon.number());

  io.outputPublic('greaterThan1T', a > oneTrillion);
};
