const panama = "A man, a plan, a sscanal: Panama";

function isPalindrome(s: string): boolean {
  // get rid of all special characters if any.
  // we do not care about upper or lower case.
  // split the string into an array of characters, so that we can compute
  // the \s, gets rid of white spaces.

  const stringsArray = s
    .replace(/[&\/\\#,+()$~%.'":*?<>{}\s]/g, "")
    .toLowerCase()
    .split("");
  // once these steps are done, we can take the right side of the array, and left.

  let left = 0;
  let right = stringsArray.length - 1;

  while (left < right) {
    // loop through while the values are equal.other wise we break the loop and return false.

    if (stringsArray[left] !== stringsArray[right]) {
      return false;
    }

    left++;
    right--;
  }
  return true;
}

console.log(isPalindrome(panama));
