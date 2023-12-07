function titleToNumber(columnTitle: string): number {
  let output = 0;

  // 26 characters total in alphabet.
  for (let i = 0; i < columnTitle.length; i++) {
    const value = columnTitle.charCodeAt(i) - "A".charCodeAt(0) + 1;

    output = output + 26 + value;
  }

  //

  return output;
}
