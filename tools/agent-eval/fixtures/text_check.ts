// Checks the agent's text.ts (run from the task's folder).
import { slugify, wordCount, isPalindrome } from "./text.ts";
const ok = slugify("  Hello, World! 2026 ") === "hello-world-2026" && slugify("--a__b--") === "a-b"
  && wordCount("one  two\tthree\nfour") === 4 && wordCount("   ") === 0
  && isPalindrome("A man, a plan, a canal: Panama") && !isPalindrome("Nuclear");
if (!ok) { console.error("wrong results"); process.exit(1); }
console.log("ok");
