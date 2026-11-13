# Set up the AI services

NuclearLang comes with two services that run on your machine: the **Smart Graph** (what an agent needs to know about
NuclearLang and Dragonfly, and the MCP server your agent talks to) and the **Nuclear Developer** (small local models
that plan, write, test and review code). Their set-up reads your machine, suggests which models fit it, tells you how
far each choice works, and installs what you choose. Nothing is downloaded before you say so.

## 1. Start it

| Where | How |
|---|---|
| The installer | `curl -fsSL https://nuclearlang.github.io/install.sh \| sh` asks at its end, in the terminal |
| VS Code | the offer the first time the extension starts, or *Nuclear: Set Up the AI Services* in the command palette, or *Set-up…* in *Settings and integrations* (`Ctrl+Alt+S`) |
| A terminal, any time | `nuclear-loop setup` |

The same steps run in the three. Run it again whenever you want another choice: what is installed is kept.

## 2. The steps

1. **This machine.** The processor, the memory, the GPU and the free disk, and where the agents run: on this machine
   (suggested: local agents write the code, so your remote agent spends tokens only on leading and correcting) or
   remote agents only.
2. **The profile.** Which models are downloaded. One is suggested for your memory and disk; the ones that are tight
   or do not fit are marked, with the reason. The choice is yours.
3. **What it needs.** Each program is found by itself once installed, or installed for you in a terminal you see:

   | Program | For | Installed for you with |
   |---|---|---|
   | NuclearLang | the compiler, `dragonfly`, the code of the services | its installer |
   | [uv](https://docs.astral.sh/uv/) | the Python environments of the services (it brings its own Python) | its installer, in your user folder |
   | [Ollama](https://ollama.com) | running the local models | its own installer on Linux (it asks for your password), Homebrew on macOS, `winget` on Windows |

   Without Ollama there are no local agents: install it, or go back and choose the remote agents only.
4. **Install.** The Smart Graph and the MCP server, the Nuclear Developer, the models, the knowledge base loaded
   into the graph (about two minutes the first time), and both services started. A step that fails says why and is
   tried again from there.

## 3. The profiles

| Profile | Models | Downloads | Memory | Measured with the Developer's evaluation |
|---|---|---|---|---|
| `lite` | `qwen2.5-coder:1.5b` | 1.6 GB | 8 GB (4 GB at least) | not measured yet |
| `medium` | `qwen2.5-coder:3b` | 2.5 GB | 12 GB (8 GB at least) | as coder, the model of hard: 43 of 45 single-file tasks, 16 of 18 never seen; as assistant, 6 of the first 8 chat questions (hard: 10 of 10) |
| `hard` | `qwen2.5-coder:3b` + `gemma4:e2b` | 9.7 GB | 24 GB (16 GB at least) | 43 of 45 single-file tasks (Claude Sonnet: 39), 16 of 18 never seen, 2 of 3 whole features, 10 of 10 chat questions, 10 of 11 analyses |
| `remote` | none | nothing but the MCP server (33 MB) | any | nothing runs on the machine |

Every local profile also downloads `embeddinggemma` (0.6 GB, counted above), which makes the graph's vectors. No model is larger than 3B parameters of effective size.

**`lite`: One 1.5B model does everything: for a machine with little memory.**

- Works: the Smart Graph: the strategy of each request and the knowledge base, for the local agents and for your remote agent.
- Works: the generators and the compiler's fixes, which need no model.
- Does not, or less: not measured yet: its model is half the size of the coder the project measures, so expect it to miss more and your remote agent to correct more.
- Does not, or less: plans, reviews and chat answers come from a small model made for code.

**`medium`: The coder the project measures does everything, also the plans and the reviews: one 3B model in memory.**

- Works: the Smart Graph: the strategy of each request and the knowledge base, for the local agents and for your remote agent.
- Works: the generators and the compiler's fixes, which need no model.
- Works: a file is written as in hard: 43 of 45 single-file tasks (Claude Sonnet: 39), 16 of 18 never seen before.
- Does not, or less: plans, reviews and chat answers come from a model made for code: 6 of the first 8 chat questions as required, where hard answers 10 of 10.
- Does not, or less: some answers are slow: one chat question had not ended after five minutes.
- Does not, or less: turning a request in plain words into generator calls is not measured yet with this model.

**`hard`: The set every measurement of the project was made with: a coder and a separate model to plan, review and answer.**

- Works: the Smart Graph: the strategy of each request and the knowledge base, for the local agents and for your remote agent.
- Works: the generators and the compiler's fixes, which need no model.
- Works: 43 of 45 single-file tasks (Claude Sonnet: 39), 16 of 18 never seen before.
- Works: the assistant answers 10 of 10 questions about NuclearLang as required, and reads 10 of 11 requests right.
- Does not, or less: its assistant model takes 7.2 GB: with 16 GB of memory little else stays open.
- Does not, or less: a whole feature of several files passes 2 times of 3: your remote agent validates and corrects.

The set-up suggests the heaviest profile your machine runs well. A profile marked *tight* loads and works with
little else open; one marked *not for this machine* may still be chosen, and will be slow or fail to load.

In a terminal, without questions: `nuclear-loop setup --profile medium --yes`.

## 4. Remote agents only

Nothing runs on your machine and nothing is downloaded but the MCP server. Claude Code, Codex, Cursor or OpenCode
get the generators, the compiler, the documentation of every word and the `dragonfly` commands as tools
([the MCP server](mcp.md)). What you give up: the Smart Graph (no strategy for a request, no search of the knowledge
base), the Loop panel and the *Nuclear agent* chat of the editor, and the local agents themselves, so your remote
agent writes every line the generators do not and all of it is paid tokens.

## 5. After the set-up

The services start at every login and are started again if they fail:

| | Linux | macOS | Windows |
|---|---|---|---|
| What starts them | two systemd user units | two launch agents | a script of the Startup folder |
| State | `systemctl --user status nuclear-graph nuclear-developer` | `launchctl print gui/$UID/io.github.nuclearlang.graph` | `nuclear-loop setup --status` |
| Restart | `systemctl --user restart nuclear-developer` | `launchctl kickstart -k gui/$UID/io.github.nuclearlang.developer` | sign out and in |
| Their output | `journalctl --user -u nuclear-graph -n 50` | `~/Library/Logs/nuclear-graph.log` | |

`nuclear-loop setup --status` says on every system what is installed and running. In VS Code, *Settings and
integrations* shows the same and restarts a service with a button.

| What | Where |
|---|---|
| Your choice (the mode, the profile, its models) | `~/.config/nuclearlang/ai.env` (`%APPDATA%\nuclearlang\ai.env`) |
| The code of the services | `~/.local/share/nuclearlang/ai/` (`%LOCALAPPDATA%\nuclearlang\ai\`) |
| The graph's data, encrypted | `~/.local/share/nuclear-graph/` |
| The models | Ollama's own folder (`ollama list`) |

In idle the two services take about 150 MB of memory. A model is loaded by Ollama when an agent asks and freed a few
minutes later.

To give your agent the MCP server: the buttons of *Settings and integrations*, or
`claude mcp add --scope user nuclear -- nuclear-graph mcp` ([every client](mcp.md#1-set-up)).

## 6. Update and remove

- **Update:** run the installer again. It brings the new version and sets the services up again with the profile
  you had, without questions.
- **Another profile:** run the set-up again and choose it. The models of the old one stay in Ollama
  (`ollama rm <model>` frees their disk).
- **Remove:** `nuclear-loop setup --remove` takes the services away; the installer's `--uninstall` does it too.
  Ollama, its models and the graph's data stay: they are yours.
- **In a script or CI:** `--no-ai` skips the set-up, `--profile <id>` takes that profile without questions, and
  without a terminal nothing is asked or downloaded.

## 7. When a step fails

| It says | What to do |
|---|---|
| `a Nuclear Smart Graph started by hand already answers at ...` | a graph you started in a terminal holds the port: stop it with the command the message gives, then *Try again* |
| `Ollama refused to pull ...`, or a model stays at the same percent | check the disk and the connection; the download goes on from where it stopped when the step is tried again |
| `no Python 3.12 or newer, and uv could not install one` | install Python 3.12 or newer and try again |
| `the Nuclear Developer does not answer at ...` | its output (the table above) says why; most often a model of the profile is missing: `nuclear-loop setup --status` |

## 8. From the source tree

`./install.sh --ai` runs the same set-up over the tree's own folders: the graph is installed editable and the
Developer is built in place, so a change to either is live after a restart of its service. After changing the
knowledge base: `nuclear-graph knowledge build && nuclear-graph ingest`.
