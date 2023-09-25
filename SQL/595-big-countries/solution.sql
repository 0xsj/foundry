  -- TRY 1
  SELECT name, population, area from World WHERE area >= 3000000 OR population >= 25000000;

  -- TRY 2
SELECT name, population, area 
FROM World 
WHERE (area >= 3000000 AND population >= 25000000)
OR (area >= 3000000)
OR (population >= 25000000);
