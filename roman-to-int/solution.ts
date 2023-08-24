function romanToInt(s: string): number {
  const romanToIntMap: Map<string, number> = new Map([
    ["I", 1],
    ["V", 5],
    ["X", 10],
    ["L", 50],
    ["C", 100],
    ["D", 500],
    ["M", 1000],
  ]);

  let result: number = 0;
  let prevValue: number = 0;

  for (let i: number = s.length - 1; i >= 0; i--) {
    const currentRoman: string = s[i];
    const currentInt: number | undefined = romanToIntMap.get(currentRoman);

    if (currentInt !== undefined) {
      if (currentInt < prevValue) {
        result -= currentInt;
      } else {
        result += currentInt;
      }
      prevValue = currentInt;
    }
  }

  return result;
}

console.log(romanToInt("IV"));
