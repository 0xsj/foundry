function gcdOfStrings(str1: string, str2: string): string {
  // immediate check for the concat
  if (str1 + str2 !== str2 + str1) {
    return "";
  }

  // calculate the greatest common divisor (GCD) of a and b
  function gcd(a: number, b: number): number {
    while (b) {
      const temporaryVar = b;
      b = a % b;
      a = temporaryVar;
    }
    return a;
  }
  return str1.slice(0, gcd(str1.length, str2.length));
}
