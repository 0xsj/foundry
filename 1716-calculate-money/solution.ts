// function totalMoney(n: number): number {
//   let week = 7;
//   let total = 0;

//   for (let i = 0; i < n; i++) {
//     let count = 0;

//     // when nums[i] % 7 === 1

//     // every 4th mondays,

//     // nums[]
//   }

//   return total;
// }

function totalMoney(n: number): number {
  let total = 0;
  let current = 1;
  let monday = 1;

  while (current <= n) {
    total += monday;
    current++;
    monday++;

    if (current % 7 === 1) {
      monday = Math.floor(current / 7) + 1;
    }
  }

  return total;
}

console.log(totalMoney(20));
// 0, 1, 2, 3, 4, 5, 6
// 7, 8, 9, 10, 11, 12,13
// 14,

/**
 * if its 20 days
 *
 * 7 m - sun
 * 7 m - sun
 * 6 m - sat
 */

/**
 *
 */
