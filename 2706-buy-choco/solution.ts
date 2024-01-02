// function buyChoco(prices: number[], money: number): number {
//   const seen = new Set<number>();
//   let total = money;

//   for (const price of prices) {
//     const complement = money - price;

//     if (seen.has(complement)) {
//       return 0;
//     }

//     if (price < money) {
//       seen.add(price);
//       total = Math.min(total, money - price);
//     }
//   }

//   return total;
// }

// console.log(buyChoco([3, 2, 3], 3));
// console.log(buyChoco([1, 2, 2], 3));

function buyChoco(prices: number[], money: number): number {
  let min = Infinity;
  let secondMin = Infinity;

  for (let i = 0; i < prices.length; i++) {
    if (min > prices[i]) {
      secondMin = min;
      min = prices[i];

      continue;
    }

    if (secondMin > prices[i]) {
      secondMin = prices[i];
    }
  }

  if (min + secondMin > money) {
    return money;
  }

  return money - (secondMin + min);
}
