function minTimeToVisitAllPoints(points: number[][]): number {
  let minTime = 0;

  for (let i = 1; i < points.length; i++) {
    const current = points[i - 1];
    const next = points[i];

    const distanceX = Math.abs(next[0] - current[0]);
    const distanceY = Math.abs(next[1] - current[1]);

    const maxDistance = Math.max(distanceX, distanceY);

    minTime += maxDistance;
  }

  return minTime;
}

function minTimeToVisitAllPoints2(points: number[][]): number {
  let total = 0;
  for (let i = 0; i < points.length - 1; i++) {
    const dx = Math.abs(points[i + 1][0] - points[i][0]);
    const dy = Math.abs(points[i + 1][1] - points[i][1]);

    total += Math.max(dx, dy);
  }

  return total;
}
