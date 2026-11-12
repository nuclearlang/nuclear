---
layer: L4
component: nuclear-developer
read-when: you train the local coder's weights (LoRA), or decide its base model, dataset or hardware
---

# 007 — A LoRA of the local coder trained on Nuclear and Dragonfly

Goal: a model of at most 3B parameters (the user's rule) that writes Nuclear and Dragonfly as its first language, so
the local developer passes whole features, not only single files. Today the coder is `qwen2.5-coder:3b` taught by
retrieval (the Smart Graph's `nuclear` collection, prompts, distilled examples): at Sonnet's level on single files,
behind it on features (`nuclear-developer/eval/results/REPORT.md`).

## What is needed

| Need | Today | For a strong LoRA |
|---|---|---|
| Training pairs (task → validated `.nuc`) | 39 validated reference answers, 26 graph examples, ~280 `.nuc` files of the tree (~4k lines) | 5 000–10 000 pairs over every kind of the three platforms, single files and features |
| Repair pairs (code + compiler errors → fixed code) | the eval's traces | 1 000–2 000: they teach the check-and-repair loop the Developer runs |
| Hardware | Radeon 880M iGPU (no ROCm support for its gfx1150), 15 GB shared memory: CPU training takes days per round | one NVIDIA GPU with 12–24 GB (QLoRA of a 3B in 1–3 h per epoch), or a rented A10/L4/A100 (≈ USD 1–2 per hour, ≈ USD 5–15 per run) |
| Tooling | — | Unsloth or HF PEFT + TRL (SFT), then `llama.cpp` to GGUF and an Ollama `Modelfile` (`nuclear-coder:3b`) |
| Measure | `eval/run.py` (45 tasks, 18 held out) | the same bank grown to ~300 tasks, the held-out split never trained on |

## How the dataset is made (the compiler is the judge)

1. **A task generator, not hand-written tasks.** The platforms already describe every kind (`docs.toml`, `[layout]`,
   `[skeletons]`, the briefs): a script crosses kinds × business domains (library, shop, clinic, school…) × variations
   (fields, relations, rules, errors) into thousands of task texts with acceptance texts.
2. **Answers validated by NuclearLang itself.** Every answer must pass `nuclear check` (as a project for features) and
   its acceptance texts; nothing else enters the dataset. Wrong answers with their errors become repair pairs.
3. **Where the answers come from — to decide with the user:**
   - *Self-distillation* (no external terms): the local model samples many answers per task, the compiler keeps the
     ones that pass, and those train the next round (rejection sampling). Free, slower to start.
   - *A frontier model*: Claude answers the tasks (≈ USD 0.02–0.03 per answer with Sonnet: ≈ USD 100–300 for 5–10k;
     the user approved USD 20 so far). **Before using Claude's answers as training data, check Anthropic's terms for
     training models on outputs** (the commercial terms restrict building competing models).
   - *The tree's own code*: every example, test and generated docs (`.nuclear/map.md` descriptions → their code).
4. **Only Nuclear in the data.** No target language (Python, TypeScript, Dart) ever appears in a pair: the model
   learns Nuclear and the platforms, the transpiler does the rest.

## The base model — to verify before training

- `qwen2.5-coder:3b` is what the Developer runs; check its license for training and redistribution (the 3B size of
  Qwen2.5 has had a different license from the 1.5B and 7B ones).
- Alternatives within 3B with permissive licenses to compare on the eval: `qwen2.5-coder:1.5b`, `llama3.2:3b`,
  `smollm2:1.7b`, `gemma` 2B-class models.

## Order

1. The task generator and the compiler-validated pipeline (no GPU): grow the eval bank to ~300 tasks first.
2. Self-distillation round 1 on CPU-sized data (a few hundred pairs) to test the pipeline end to end.
3. A rented GPU run (QLoRA, 2–3 epochs), GGUF, Ollama; the eval against the retrieval-only coder.
4. Keep retrieval on top of the LoRA (the graph still gives the project's own names and the map).

Done when the LoRA coder passes the held-out features the retrieval-only coder fails, with no loss on single files.
