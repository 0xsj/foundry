function findDifference(nums1: number[], nums2: number[]): number[][] {
  const set1 = new Set(nums1);
  const set2 = new Set(nums2);

  const difference1 = [...set1].filter((num) => !set2.has(num));
  const difference2 = [...set2].filter((num) => !set1.has(num));

  return [difference1, difference2];
}
//
function findDifference_bubble(nums1: number[], nums2: number[]): number[][] {
  const bubbleSort = (arr: number[]) => {
    const n = arr.length;
    let swapped;
    do {
      swapped = false;
      for (let i = 0; i < n - 1; i++) {
        if (arr[i] > arr[i + 1]) {
          const temp = arr[i];
          arr[i] = arr[i + 1];
          arr[i + 1] = temp;
          swapped = true;
        }
      }
    } while (swapped);
  };

  const clonedNums1: number[] = [...nums1];
  const clonedNums2: number[] = [...nums2];

  bubbleSort(clonedNums1);
  bubbleSort(clonedNums2);

  const difference1: number[] = [];
  const difference2: number[] = [];

  let i = 0;
  let j = 0;

  while (i < clonedNums1.length || j < clonedNums2.length) {
    if (i < clonedNums1.length && (j >= clonedNums2.length || clonedNums1[i] < clonedNums2[j])) {
      difference1.push(clonedNums1[i]);
      while (i < clonedNums1.length - 1 && clonedNums1[i] === clonedNums1[i + 1]) {
        i++;
      }
      i++;
    } else if (
      j < clonedNums2.length &&
      (i >= clonedNums1.length || clonedNums2[j] < clonedNums1[i])
    ) {
      difference2.push(clonedNums2[j]);
      while (j < clonedNums2.length - 1 && clonedNums2[j] === clonedNums2[j + 1]) {
        j++;
      }
      j++;
    } else {
      while (i < clonedNums1.length - 1 && clonedNums1[i] === clonedNums1[i + 1]) {
        i++;
      }
      i++;
      while (j < clonedNums2.length - 1 && clonedNums2[j] === clonedNums2[j + 1]) {
        j++;
      }
      j++;
    }
  }

  return [difference1, difference2];
}

// Example usage:
const nums1 = [1, 2, 3, 3];
const nums2 = [1, 1, 2, 2];
const result = findDifference_bubble(nums1, nums2);
console.log(result); // Output: [[3], []]
