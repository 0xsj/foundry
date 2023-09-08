function canPlaceFlowers_brute(flowerbed: number[], n: number): boolean {
  let i = 0;
  let count = 0;

  while (i < flowerbed.length) {
    if (
      flowerbed[i] === 0 &&
      (i === 0 || flowerbed[i - 1] === 0) &&
      (i === flowerbed.length - 1 || flowerbed[i + 1] === 0)
    ) {
      flowerbed[i] = 1;
      count++;
    }
    i += 2;
  }

  return count >= n;
}

function canPlaceFlowers_(flowerbed: number[], n: number): boolean {
  const len = flowerbed.length;
  let existingFlowers = 0;

  // Calculate the number of existing flowers
  for (let i = 0; i < len; i++) {
    if (flowerbed[i] === 1) {
      existingFlowers++;
    }
  }

  // Calculate the maximum number of flowers that can be placed
  const maxFlowers = Math.floor((len + 1) / 2) - existingFlowers;

  return maxFlowers >= n;
}
