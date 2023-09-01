const prices1 = [1, 2, 3, 4, 5, 6];
const prices2 = [7, 1, 5, 6, 4, 3];
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
