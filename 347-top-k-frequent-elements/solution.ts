/**
 * 1. the idea here is to distribute the elements of an array into a set number of "buckets" or containers
 * 2. create a frequencyMap, where we are going to store the frequency of each number in the input array
 * 3. we iterate through the nums array
 * 4. if num is already in the map, we retreive it. otherwise we default to 0. this ensures we can increment through the frequency even if num is not present
 *    increment the frequency by 1, indicating that we encountered num one or more time in the input array
 * 5. we create an array of buckets - where each bucket represents a frequency count.
 * 6. we go through the frequencymap entries and group numbers with the same frequency into their respective buckets
 * 7. we store the results in an array
 * 8. go through the bucket in descending order of frequency, push the numbers from the current bucket into the result array.
 * 9. return the slice starting at the beginning, and ending at k frequent numbers.
 */
function topKFrequent(nums: number[], k: number): number[] {
  const frequencyMap = new Map();

  console.log("after initializiation: ", frequencyMap);

  for (const num of nums) {
    console.log("value of num", num);
    frequencyMap.set(num, (frequencyMap.get(num) || 0) + 1);
  }

  console.log("after map.set()", frequencyMap);

  const buckets: number[][] = new Array(nums.length + 1).fill(null).map(() => []);

  console.log("bucket:", buckets);
  console.log("map entries:", frequencyMap.entries());

  for (const [num, frequency] of frequencyMap.entries()) {
    buckets[frequency].push(num);
  }

  const result: number[] = [];
  console.log("results empty", result);

  for (let i = buckets.length - 1; i >= 0 && result.length < k; i--) {
    if (buckets[i].length > 0) {
      console.log("buckets[i]", buckets[i]);
      result.push(...buckets[i]);
      console.log("results in for loop", result);
    }
  }

  return result.slice(0, k);
}

console.log(topKFrequent([1, 2, 3, 4], 2));
console.log("================================================================================ \n");
console.log(topKFrequent([1, 1, 1, 2, 2, 3], 2));
console.log("================================================================================ \n");
console.log(topKFrequent([1], 1));
