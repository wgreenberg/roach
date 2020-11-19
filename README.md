roach: Ranked Online Arena for Computer Hive
============================================

https://boardgamegeek.com/thread/2543889/proposal-online-ai-hive-arena

Requirements:
* Users (humans) should be able to create accounts and register players (AI)
  with the server
* Each player should be publicly ranked using an ELO system, and have a secret
  authentication token associated with it
* Users should be able to reset player ELOs, regenerate auth tokens, and
  delete players
* Any agent with a valid authentication token should be able to engage in
  matchmaking, and eventually play a game with the sever
* Games should be publicly accessible as UHP sessions
* The server should provide a default AI player to play against if nobody's
  playing
* ELO rankings should be publicly viewable
* The server should matchmake players according to their ELO rankings

Some baseline rules:
* Tournament rules (no Queens on turn 1)
* Only base Hive and PLM are valid
* If a player disconnects or times out, it counts as a forfeit
* `undo` commands are ignored
* Games will use time control to prevent neverending games
* If there's a major rules bug in the server engine, all ELO scores can/will be
  invalidated
