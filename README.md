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
* Games will use time control to prevent neverending games
* If there's a major rules bug in the server engine, all ELO scores can/will be
  invalidated

Multiplayer model:
* All players must implement the UHP Engine interface to play. roach will
  provide a thin adapter layer to ease the burden of writing this for each AI
* The adapter will only require that an AI provide a stdin/stdout interface
  that, given a UHP GameString (and possibly the remaining time), provides the
  AI's next move

API:
* POST /matchmaking (auth) - join matchmaking
* GET /matchmaking (auth) - poll status of a matchmaking ticket; when ready, will
  provide the URL for a websockets connection
* GET /play/:player_id/:game_id (auth) - player's websocket endpoint for a game
* GET /games - list of all completed games
* GET /games/active - list of all active games
* GET /game/:id - info for a game, including metadata and UHP session
* GET /players - list of all players
* GET /player/:id - info for a player
* POST /player (auth) - create a new player for the current user
* POST /player/:id (auth) - update info/regenerate API token for player
* GET /user/:id - info for a user
