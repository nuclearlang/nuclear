---
adr: 0099
title: "The Nuclear workspace in the editor: projects as tabs, tools full screen, keyboard first"
status: "Accepted"
scope: editors
---

# ADR-0099: The Nuclear workspace in the editor: projects as tabs, tools full screen, keyboard first

- Status: Accepted
- Scope: `editors` (the VS Code extension first; Android Studio and Zed follow where their platform allows)
- Date: 2026-11-06 (asked by the user, with a sketch of the layout)

## Context

Every Nuclear feature in the editor was its own view: the HTTP client in one activity bar container, the device
in a panel, the projects as VS Code windows. Working on a backend and its apps meant several windows, or a sidebar
cut into small sections that steal space from the code. The user wants one place where everything of Nuclear meets,
inspired by tiling window managers: each project and each tool is a tab, the whole screen goes to one thing at a time,
and the keyboard moves between them; VS Code's own explorer, search and git are left as they are unless the user
wants them.

## Decision

1. **One container.** The extension contributes a single activity bar container, `Nuclear` (the trefoil mask),
   with one *Workspace* webview view. No other Nuclear container or sidebar view is added: the sidebar is the strip
   and a pane that depends on the active tab, the editor area is for one thing at a time.
2. **The strip.** The workspace view is a vertical strip of tabs beside the pane: the projects at the top (the Nuclear
   projects of the workspace folders, then the ones opened as tabs, an empty `+` slot to open more; each tab shows the
   platform's glyph with the dialect's icon as a badge: mobile + flutter, web + react, backend + python or nest) and
   the tools at the bottom (HTTP client, devices, UI designer, AI documenter, Nuclear agent, settings and
   integrations). A tool that is not built yet has a template page that says what it will do; the layout and its
   shortcut come first, the tool later. **Every tab is a workspace of its own**, a project or a tool: switching swaps
   the editor area (the editors of the tab left are remembered per window and closed, those of the tab entered
   reopened). The exception is the devices: they stay beside the code in the **mobile** workspaces and the Devices
   tool, like emulators, because hot reload is watched while coding; entering a backend, web or other tool workspace
   parks them (the panel closes, the app keeps running headless with its session) and the next mobile workspace gives
   them a panel again, side by side. Every workspace switch parks the devices and re-adopts them *after* the code is
   reopened, so each device lands in its own locked editor group beside the code (the right, like Android Studio) and
   is never merged into the code's tab group; a device streams at its full `dpr` (physical pixels), so the 3x phones
   stay sharp on a HiDPI screen.
3. **A project is a tab, not a window, and shares nothing with the others.** *Open a project as a tab* remembers the
   folder (user state) and shows its files in the workspace view; it does not touch the VS Code workspace folders. One
   command adds the active project's folder to them when the user wants the explorer, search and git on it. **The
   actions of its platform sit in the editor title bar** (the row of the editor tabs, as a toolbar; never inside the
   file tree, asked by the user), each with a shortcut (`Ctrl+Alt+R/G/L/B/T/C`) VS Code shows in its tooltip: mobile
   runs, debugs and hot reloads the app on the selected Flutter device or on a Dragonfly device, builds, tests, checks;
   a backend serves, tests, builds, checks and jumps to its routes in the HTTP tool; a web app serves, builds, tests,
   checks. Each platform has its own set: nothing of one platform is offered to another (context key
   `nuclear.platform`; no buttons while a tool is the active tab).
4. **The tools are workspaces too, independent of the projects.** The HTTP tool works like Bruno: its pane is the
   endpoints tree of every project (platform → project → contracts / components → endpoints, OpenAPI documents, the
   environment), a **component is an editor tab** and **each endpoint a tab inside it**, so the tab bar never fills
   with requests; a component panel keeps its tabs, forms and responses while the workspace is left and comes back as
   it was. Environments belong to the window (`<first folder>/.nuclear/http.json`, else the extension's storage). The
   devices tool lists the open Dragonfly devices with their state and opens as many as wanted (a preset each), like
   several emulators at once, each in its own locked editor group tiled horizontally with its own app, so several
   users can be signed in at once; a project's ▷ Device offers the open devices, a new one or the Devices workspace
   and runs on the chosen one beside the code; ▷ Run and ◉ Debug of a mobile project offer the same devices plus
   *Other Flutter device…* (Flutter's own picker: emulators, phones, Chrome, desktop); a run that names no device takes
   a free one or opens a new one, never a running one; a device's ▷ run restarts its own app. **An app lives on its
   own:** closing a device tab only hides it (the app and its session go on; the Devices workspace lists it as hidden
   and shows it again), the ✕ of the Devices workspace, the panel's ■ or the end of its session end it at once
   (session, page connection, browser, Flutter); a watchdog runs every minute: a device nobody looked at for 30
   minutes ends, and any Flutter or browser process of this window that no device owns is killed. Devices take the
   lowest free number, so a closed one frees its name; a session is named after its device (the debug toolbar and the
   Debug Console pick by device). The strip shows the tools with glyphs in the theme's `nuclearHttp.*` colors. The
   other tools open a full-screen template page.
5. **The file tree is the explorer's equal.** It wears the Chernobyl icon theme (the user's variant; the same lookup
   as VS Code: file name, longest extension, folder name, open or closed) and hides what `files.exclude` hides.
6. **Keyboard first.** Fixed bindings: `Ctrl+1` … `Ctrl+9` the project tabs, `Ctrl+Alt+N` the container,
   `Ctrl+Alt+O` open a project, `Ctrl+Alt+W` the welcome page, and one `Ctrl+Alt+<letter>` per tool (`H`, `D`, `M`,
   `A`, `S`). Clicking a tab does the same and shows a short toast with the shortcut, so the mouse teaches the keys.
   The user may rebind any of them.
7. **A welcome page at startup** (`nuclear.workspace.welcome`, on by default, only when the window restored no editor)
   explains the workspace, lists the projects with their shortcut and the tools, and can turn itself off.
8. The other editors follow as far as they can: Android Studio tool windows and actions with shortcuts (the HTTP tool
   window with `Alt+Shift+H` exists since 0.30.0); Zed cannot contribute keybindings or views.

9. **Every platform runs, debugs and reloads from the toolbar, pre-configured (2026-11-10, editors 0.37, asked by the
   user: "validate that every runner, debug and script is available and pre-configured for each platform, as with
   flutter").** mobile: ▷ Run, ◉ Debug, ↻ Reload, ▭ Device, Build, Test, Check (unchanged). backend: ▷ Serve
   (`dragonfly run`; a TypeScript server restarts on each build with `node --watch`, fastapi with `uvicorn --reload`),
   ◉ Debug (debugpy for fastapi, node with source maps for nestjs), Test, Build, Check, ⇄ API. web: ▷ Serve
   (`dragonfly run`, Vite), ▭ Device, ◉ Debug, ↻ Reload, Build, Test, Check. Saving a `.nuc` of a backend or web
   project builds it, so the running server or page reloads (`nuclear.buildOnSave`); `install` runs first when the
   app has no packages. `dragonfly build` writes `build/<name>.code-workspace` for every platform (tasks build,
   install, run, test, check; the dialect's debug launch), and every `nuclear build` writes `.nuclear/lines.json`,
   so a breakpoint in a `.nuc` stops in every debugger and the paused line shows in the `.nuc`.
10. **The web device (2026-11-10, editors 0.37, asked by the user: "a device like mobile, customized for web, with what
    developers need").** ▭ Device of a web project opens the app beside the code in a headless Chromium the editor
    drives (the mobile device's CDP machinery): desktop, laptop, tablet and phone sizes (user agent and touch for
    tablets and phones), rotation, *Responsive* with any size, zoom (always sharp); an address bar with back, forward,
    reload (Shift: without cache) and the app's address, the page in the user's browser, Chrome's DevTools of the
    page; `prefers-color-scheme`, network throttling (fast 4G, slow 3G, offline), no cache; a dock with the page's
    Console (errors counted, a line's source opens the generated file) and Network (method, status, address, type,
    size, time), filtered; a screenshot into `build/screenshots/`; ◉ Debug attaches VS Code's JavaScript debugger to
    that page.

## Consequences

- Easier: one place for every Nuclear feature; projects side by side without windows; the screen for one thing at a
  time; every action has a key.
- Limits: the HTTP tree is drawn by the webview (its own glyphs and the `nuclearHttp.*` colors), not a native tree;
  a component panel's memory lives in the window (not on disk); the file tree of the workspace view is the
  extension's (a webview), not VS Code's explorer: no git decorations, no drag and drop, no context menu with
  new/rename/delete (the user adds the folder to the VS Code workspace for those); the editors remembered per
  workspace are text editors (an unsaved editor is not closed by a switch, so no save prompt: it stays open until saved); the Dart extension launches Flutter only for a project inside the workspace folders ("Dart-only
  workspace" otherwise), so Run/Debug/Device of a mobile project opened as a tab first adds its folder to them (the
  window reloads once, user state `nuclear.workspace.pendingAction` continues the action); the editor title bar is VS Code's only toolbar an extension can
  fill, so the actions show only while an editor is open (not on the welcome page); the terminals the actions open are
  named per project and action but not swapped; the web actions call `nuclear build .` and `npm --prefix build/app`
  until the `dragonfly` CLI has web projects; `Ctrl+1` … `Ctrl+9` replace VS Code's "focus editor group N"; `Ctrl+Alt+<letter>` may be taken
  by a desktop environment (the user rebinds); the tools other than HTTP and devices are templates.

## Alternatives rejected

- **Multi-root workspace folders as the project tabs**: the first added folder reloads the window and VS Code's
  explorer shows every folder at once; the user asked not to touch VS Code's own system unless wanted.
- **The native explorer beside the strip**: a built-in view cannot be placed inside an extension's container, and
  the views of a container stack vertically, so the strip and a tree side by side need one webview.
- **A horizontal tab row** above a native tree: fits the sidebar's stacking, but not the vertical strip the user drew.
