---
adr: 0106
title: The AI services come with the installer: a set-up in steps, a profile for the machine, the services at every login
status: Accepted
scope: nuclear-vscode
---

# ADR-0106: The AI services come with the installer

- Status: Accepted (asked by the user, 2026-11-13: "I need this built into the installer: every service the extension
  brings installed locally"; "the extension analyses the hardware and offers lite, medium and hard, next-next style,
  the best option for the hardware suggested, the user decides what is downloaded, and we warn how far the service
  works and where it does not"; the services start at login; "look for a better option than Ollama; if there is none,
  suggest Ollama, detect when it is installed, offer to install it ourselves as one of the steps; a user who does not
  want it chooses the remote agents alone, and we suggest the local option to save tokens")
- Scope: `nuclear-vscode` (the set-up's engine, the wizard, `nuclear-loop setup`), `nuclear` (the installers, the
  release archive), `nuclear-graph` (the MCP server for remote agents only)
- Date: 2026-11-13

## Context

The Smart Graph and the Nuclear Developer were installed from the source tree only (`./install.sh --ai`), followed by
nine commands typed by hand: pull three models, start the graph, build and ingest the knowledge base, connect two
clients, start the Developer. The release installers brought `nuclear-loop` and nothing else. The services were
processes started by hand: after a restart of the machine nothing answered. And there was one set of models for every
machine: 9.7 GB, of which the assistant's model is 7.2 GB and leaves a 16 GB machine with under 1 GB of memory.

## Decision

1. **Every release archive carries the code of the services** (`ai/nuclear-graph`, `ai/nuclear-developer` with its
   knowledge base: 5 MB). What is large is downloaded by the set-up, after the user chose: the Python packages
   (≈ 550 MB) and the models. Nothing of a model fits a release (a release file is at most 2 GB).

2. **One set-up, two fronts.** `aiSetup.ts` reads the machine, finds what is installed and runs the steps. The editor
   shows it as a wizard (*Nuclear: Set Up the AI Services*; offered once on a machine never set up; a button in
   *Settings and integrations*); a terminal as `nuclear-loop setup`, which the installers call at their end. Nothing
   is downloaded before the user presses Install or answers the question.

3. **The wizard's steps**: this machine (processor, memory, GPU, free disk) and where the agents run; the profile;
   what it needs; the installation, with the progress of each step and *Try again* from the step that failed.

4. **Profiles are data** (`aiSetup.json`): `remote`, `lite`, `medium`, `hard`. Each names its models, what it
   downloads, the memory it runs well with and the least it loads with, what works with it and what does not. The
   set-up suggests the heaviest that fits the machine and marks the ones that are tight or do not fit, with the
   reason; the user chooses, also one that does not fit. What each profile says of itself is measured with the
   Developer's own evaluation (`eval/run.py`), not estimated:

   | Profile | Models | Downloads | Memory | Measured with the Developer's evaluation |
   |---|---|---|---|---|
   | `lite` | `qwen2.5-coder:1.5b` | 1.6 GB | 8 GB (4 GB at least) | not measured yet |
   | `medium` | `qwen2.5-coder:3b` | 2.5 GB | 12 GB (8 GB at least) | as coder, the model of hard: 43 of 45 single-file tasks, 16 of 18 never seen; as assistant, 6 of the first 8 chat questions (hard: 10 of 10) |
   | `hard` | `qwen2.5-coder:3b` + `gemma4:e2b` | 9.7 GB | 24 GB (16 GB at least) | 43 of 45 single-file tasks (Claude Sonnet: 39), 16 of 18 never seen, 2 of 3 whole features, 10 of 10 chat questions, 10 of 11 analyses |
   | `remote` | none | nothing but the MCP server (33 MB) | any | nothing runs on the machine |

   Every local profile also downloads `embeddinggemma` (0.6 GB, counted above), which makes the graph's vectors. No model is larger than 3B parameters of effective size.

   `lite` and the assistant's part of `medium` are not fully measured yet: the wizard says so in their cards.

5. **What the services need is data too**: NuclearLang itself, `uv` (the Python environments; it brings its own
   Python) and Ollama (the local profiles only). Each is found by its command, or installed by its own installer in a
   terminal the user sees (Ollama's asks for a password on Linux), or by hand from its page: the wizard checks every
   three seconds and sees it once it is there.

6. **Ollama stays the program that runs the models.** Looked at on 2026-11-13: llama.cpp's own server, LM Studio,
   MLX, vLLM. Ollama, LM Studio, Jan and others run the same engine (llama.cpp), so on the same model and quantization
   the published differences are a few percent, and the comparisons found disagree with each other; MLX is clearly
   faster on Apple silicon, and Ollama has had an MLX engine there since March 2026. What decides is the rest: one
   installer per system that leaves a service running, models pulled by name, a model loaded when asked and freed
   afterwards, chat and embeddings behind one address, an open licence, no window that must stay open. llama.cpp's
   server would be ours to package for every system and GPU, with the models' files and memory managed by us; LM
   Studio is not open and is an application; vLLM is for GPU servers. Where the models run is one address
   (`OLLAMA_URL`): changing the program later touches the Developer's connection, not the set-up.

7. **Remote agents only** is a choice of the first step: nothing runs on the machine. The MCP server is installed
   alone (`nuclear-graph[mcp]`, 33 MB instead of 405), offers the generators, the compiler, the command line and the
   internet tools, takes away the tools that need the graph or the local developer, and its instructions ask the
   model to write what the generators do not. The wizard says what that costs: every line is paid tokens.

8. **The choice is one file**: `~/.config/nuclearlang/ai.env` (`%APPDATA%\nuclearlang\ai.env`), `KEY=value` lines:
   the mode, the profile, its models. The services start with it; the Loop and the editor take their default models
   from it; the MCP server reads the mode from it. A model set in the editor's settings wins there and is written to
   the file when the editor restarts a service.

9. **The services start at every login**, with the system's own means and no administrator rights: two systemd user
   units on Linux (`nuclear-graph`, `nuclear-developer`, restarted when they fail), two launch agents on macOS, a
   script in the Startup folder on Windows. The editor's Start and Restart go through them. A system without a user
   service manager gets the services started now and is told so. A service started by hand that holds the port stops
   the set-up with the command that stops it.

10. **An update keeps the choice.** The environments live inside the installation, so an installer that replaces it
    runs the set-up again with the profile the machine has, without questions. `--no-ai` skips it, `--profile <id>`
    takes one without questions, and without a terminal (CI) nothing is asked or downloaded.

11. **Uninstalling** takes away what starts the services, the graph's tool, the Developer's launcher and the file of
    the choice (`nuclear-loop setup --remove`, called by `--uninstall`). Ollama, its models and the graph's data stay:
    they are the user's.

## Consequences

- One command installs everything: `curl -fsSL https://nuclearlang.github.io/install.sh | sh`, then the questions of
  the set-up. From VS Code alone, the wizard installs NuclearLang too.
- A machine with 8 GB gets local agents that fit it, instead of a set that does not load.
- The services survive a restart of the machine and work with the editor closed (MCP clients, `nuclear-loop`).
- From the source tree, `./install.sh --ai` is the same set-up over the tree's own folders (the graph installed
  editable, the Developer built in place).

## Limits

- The set-up is in VS Code and in a terminal. Android Studio has no wizard: `nuclear-loop setup`.
- macOS and Windows are written and not run: the launch agents, the Startup script, `winget` and Homebrew for
  Ollama. Only Linux with systemd ran.
- No GPU is recognised except NVIDIA's (`nvidia-smi`) and Apple silicon: on other machines the set-up says that no
  GPU with memory of its own was found, although Ollama may use an integrated one.
- The first load of the knowledge base asks Ollama for an embedding of every row: about two minutes.
- The models are Ollama's files in the user's folder: several users of one machine download them once each.
