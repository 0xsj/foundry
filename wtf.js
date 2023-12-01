function findHiggs(particles) {
  let closestDistance = Infinity;
  let closestIndices = [];

  for (let i = 0; i < particles.length - 1; i++) {
    for (let j = i + 1; j < particles.length; j++) {
      const distanceSquared = calculateDistanceSquared(particles[i], particles[j]);

      if (distanceSquared === closestDistance) {
        closestIndices.push(i, j);
      } else if (distanceSquared < closestDistance) {
        closestDistance = distanceSquared;
        closestIndices = [i, j];
      }
    }
  }

  return closestIndices;
}

function calculateDistanceSquared(particle1, particle2) {
  const deltaX = particle1[0] - particle2[0];
  const deltaY = particle1[1] - particle2[1];
  return deltaX * deltaX + deltaY * deltaY;
}

function findSmallestInterval(numbers) {
  numbers.sort((a, b) => a - b);

  let smallestInterval = Infinity;

  // Iterate through the sorted array to find the smallest interval
  for (let i = 1; i < numbers.length; i++) {
    const currentInterval = Math.abs(numbers[i] - numbers[i - 1]);
    smallestInterval = Math.min(smallestInterval, currentInterval);
  }

  return smallestInterval;
}
