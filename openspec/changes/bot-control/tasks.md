# Tasks

- [x] 1.1 The channel thread and the command queue (design 1): `desktop/showcase/control.rs`.
- [x] 1.2 `status`, `do`, `skip`, `skip_shot`, `pause`, `resume`, `still`, `reload` (design 2). Driven live, a stall pauses the bot instead of ending the run.
- [x] 1.3 `tools/bot_ctl.py`, the command line.
- [x] 1.4 `tools/bot_mcp.py`, the MCP server (stdio by hand), and `.mcp.json`; protocol checked (`initialize`, `tools/list`, `tools/call`).
- [x] 1.5 Hot reload: the shot file edited during a run is read again within a second and the current shot starts over on it, the bot keeping its memory (owner: "I dont want you to use too many tokens driving it via mcp, maybe some sort of hot reload path for the bot via mcp would be cool").
- [ ] 1.6 Checked live on the mission playthrough.
- [x] 1.7 Ctrl+Shift+F: the player takes the controls and gives them back (design 4).
