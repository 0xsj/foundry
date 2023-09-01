const prices1 = [1, 2, 3, 4, 5, 6];
const prices2 = [7, 1, 5, 3, 6, 4];
const prices3 = [6, 3, 2, 3, 5, 7];

// we are buying low, selling high.

/**
 * 1. initialize min and max to prices[0] and 0
 * we are assuming that first price in the array is the minimum price we seen "so far".
 * its essentially just starting points.
 * 2. iterate through the prices array, starting from the second index because min is already set to the first element.
 * 3. min update - we get the min value using math.min, passing in the element observed so far
 * 4. max udpate - same
 * 5.  return the max.
 */
const maxProfit_try1 = (prices: number[]): number => {
  let min = prices[0];
  let max = 0;

  for (let i = 1; i < prices.length; i++) {
    min = Math.min(min, prices[i]); // guaranteed to get the smallest number
    max = Math.max(max, prices[i] - min);
  }

  return max;
};

/**
 * two pointer
 * 1. set buy to 0, max to 0. buying point is representd as the beggining of the array
 * in turn, the max is set to 0, because it represents the maximum profit we observed "so far"
 * 2. set sell to 1, the index of the selling point.
 * 3. we start a while loop that continues as long as the sell index is less than the length of the prices array.
 * 4. the first if check looks for whether or not the price at the sell index is greater than the price at the buy index.
 * if this is true, we know that there is an opportunity by selling at the sell price.
 * 5. we calculate the potential profits and upadate our max. we know that the max profit is the opportunity (because the selling price is greater than the buying price)
 * 6. if the condition is false, the sell price is NOT higher than the b uy price. so there are no profits.
 * 7. increment sell until the if block is false, we return the max
 *
 * time - O(n)
 * space - O(1)
 */

const maxProfit_twoPointer = (prices: number[]): number => {
  let buy = 0; // O(1)
  let sell = 1; // O(1)
  let max = 0; // O(1)

  while (sell < prices.length) {
    // O(n)
    if (prices[sell] > prices[buy]) {
      // O(1)
      max = Math.max(max, prices[sell] - prices[buy]); // O(1)
    } else {
      buy = sell;
    }
    sell++;
  }

  return max;
};
