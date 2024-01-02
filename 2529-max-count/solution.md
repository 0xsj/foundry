## Key information

- the array is sorted in non-decreasing order

## First glance

- we can probably do this in a way where we keep two counters, "neg" and "pos"
- we can run a loop that runs, and increments pos++ and neg++ for any values that is less than or greater than 0
- we can then run a math function, that is able to take the max value between "pos" and "neg".
  IE: Math.max(pos, neg)

## Try 1: single pass

- create a neg, pos pointer
- for (let i = 0; i < nums.length; i++)
- if (nums[i] > 0) - if nums[i] is greater than 0, aka positive int, pos++
- if (nums[i] < 0) - if nums[i] is less than 0, nums++
- Math.max(neg, pos)

## Try 2: two pointer

- create a left and a right, where we are going to keep in track of the element in the array
- while(left < right)
- create a neg, pos, where we are going to keep in track of what we encounter
- return the math.max
