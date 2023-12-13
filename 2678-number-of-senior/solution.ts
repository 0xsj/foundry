function countSeniors(details: string[]): number {
  let count = 0;

  // perhaps create a new mapping / reduce the array, where we filter out each item
  // splice everything from starting position to the 11, because we care about the number after the alphabet character
  // compare the number, if num > 60, count++

  details.forEach((str) => {
    const age = parseInt(str.slice(11, 13));
    if (!isNaN(age) && age > 60) {
      count++;
    }
  });

  console.log(count);

  return count;
}

countSeniors(["7868190130M7522", "5303914400F9211", "9273338290F4010"]);
countSeniors(["1313579440F2036", "2921522980M5644"]);
