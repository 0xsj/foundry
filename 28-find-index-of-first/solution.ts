/**
 * try 1
 * 1. using javascript includes to see if needle exists in haystack
 * 2. if it does, return the indexof needle
 */
function strStr_try1(haystack: string, needle: string): number {
  if (haystack.includes(needle)) {
    return haystack.indexOf(needle);
  }
  return -1;
}

/**
 *
 */

function strStr_try2(haystack: string, needle: string): number {
  for (let i = 0; i <= haystack.length - needle.length; i++) {
    if (haystack.substring(i, i + needle.length)) {
      return i;
    }
  }

  return -1;
}

console.log(strStr_try2("hello", "ll"));
