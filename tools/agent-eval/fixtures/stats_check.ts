// Checks the agent's stats.ts (run from the task's folder).
import { mean, median, stddev } from "./stats.ts";
const near = (a: number, b: number) => Math.abs(a - b) < 1e-9;
const ok = near(mean([1, 2, 3, 4]), 2.5) && near(median([5, 1, 3]), 3) && near(median([4, 1, 3, 2]), 2.5)
  && near(stddev([2, 4, 4, 4, 5, 5, 7, 9]), 2);
if (!ok) { console.error("wrong results"); process.exit(1); }
console.log("ok");
