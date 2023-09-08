function canPlaceFlowers(flowerbed: number[], n: number): boolean {
  let count = 0;
  let i = 0;

  while (i < flowerbed.length) {
    if (flowerbed[i] === 0) {
      const prevEmpty = i === 0 || flowerbed[i - 1] === 0;
      const nextEmpty = i === flowerbed.length - 1 || flowerbed[i + 1] === 0;

      if (prevEmpty && nextEmpty) {
        flowerbed[i] = 1;
        count++;
      }
    }
    i++;
  }

  return count >= n;
}
