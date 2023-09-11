function maxArea(height: number[]): number {
  let left = 0;
  let right = height.length - 1;
  const mid = Math.floor((left + right) / 2);
  let maxArea = 0;

  while (left < right) {
    const h1 = height[left];
    const h2 = height[right];
    const width = right - left;
    const currentArea = Math.min(h1, h2) * width;

    maxArea = Math.max(maxArea, currentArea);

    if (h1 < h2) {
      left++;
    } else {
      right--;
    }
  }

  // the area is going to be the value of the said index * length - wherever you started from

  // we can maybe do all combinations and find the largest integer and return that

  return maxArea;
}
