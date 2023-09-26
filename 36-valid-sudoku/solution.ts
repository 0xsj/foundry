function isValidSudoku(board: string[][]): boolean {
  const n: number = 9;

  const hasUnique = (arr: string[]): boolean => {
    const seen = new Set<string>();
    for (const str of arr) {
      if (str !== "" && seen.has(str)) {
        return false; // Duplicate found
      }
      seen.add(str);
    }
    return true;
  };

  for (let i = 0; i < n; i++) {
    const row = board[i];
    const column = board.map((r) => r[i]);

    if (!hasUnique(row) || !hasUnique(column)) {
      return false;
    }
  }

  for (let i = 0; i < n; i += 3) {
    for (let j = 0; j < n; j += 3) {
      const box: string[] = [];
      for (let x = 0; x < 3; x += 1) {
        for (let y = 0; y < 3; y += 1) {
          box.push(board[i + x][j + y]);
        }
      }
      if (!hasUnique(box)) {
        return false;
      }
    }
  }

  return true;
}
