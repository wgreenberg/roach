Universal Hive Protocol and remote play
=======================================

# UHP 2.0

https://github.com/jonthysell/Mzinga/wiki/UniversalHiveProtocol

## New command: `newgame remote`

Instructs the game engine to create a new game (this could involve matchmaking,
and may take a while) where the semantics are different than a "normal" game.
The viewer can only make 1 ply, which the engine will then automatically reply
to, resulting in two new GameStrings (state after ply 1, state after ply 2) and
a MoveString (the ply made by the engine). The returned GameString will be an
empty new game if the viewer goes first, or a GameString w/ one move in it if
they go second

## New semantics for `play MoveString` under a `remote` game

Instructs the engine to validate/play the given MoveString, printing a
GameString if successful, and then await a response turn from another player.
When this turn is submitted (either by the Engine's AI or by another player),
the Engine will then print that MoveString and resulting GameString, and then
finally "ok", separated by a newline.

For Example:

```
play wA1
Base;InProgress;Black[1];wA1
bB1 -wA1
Base;InProgress;White[2];wA1;bB1 -wA1
ok
```

Here, white played their first Ant, and then awaited black's response. Black
then played their beetle next to white's ant, and play returned to white.

## New command: `play forfeit`

This will immediately end the game for the current player and result in a win
for the other player.

# Using UHP 1.0

Assuming we don't alter UHP in any fundamental ways, here's two models of how
remote play could work:

## Three Engine Model

In this model, the players implement the Engine interface, while the server
maintains three UHP sessions:

| Name | Viewer | Engine   |
| ---- | ------ | -------- |
| main | V      | E        |
| P1   | E      | player 1 |
| P2   | E      | player 2 |

```
       V
       |
P2 <=> E <=> P1
```

This makes sense because players should have essentially no input on gamestate
besides playing the move for their turn only, which the server solicits via the
`bestmove` command. The server should dictate options to the players' AI, such
as time control. No changes to UHP are needed.

The main session will be recorded as the canonical game record. Here's an
example of the first few moves

V->E: newgame
E->P1,P2: newgame s1
E->V: s1
V->E: bestmove
E->P1: bestmove
P1->E: m1
E->V: m1
V->E: play m1
E->P1,P2: play m1
P1,P2->E: s2
E->V: s2
V->E: bestmove
E->P2: bestmove
P2->E: m2
E->V: m2
V->E: play m2
E->P1,P2: play m2
P1,P2->E: s3
E->V: s3

V,E:
```
newgame
s1
ok
bestmove
m1
ok
play m1
s2
ok
bestmove
m2
ok
play m2
s3
ok
```

E,P1
```
newgame s1
s1
ok
bestmove
m1
ok
play m1
s2
ok
play m2
s3
ok
```

E,P2
```
newgame s1
s1
ok
play m1
s2
ok
bestmove
m2
ok
play m2
s3
ok
```

## One Engine Model

Here, the server maintains the only true UHP session, and interfaces with the
players using a simplified protocol. Players initiate a game session and get
info on what color they play, after which the server will only send each
player a stream of GameStrings. When a GameString indicates that a player
should move, the server will expect a MoveString in response, to which the
server will reply with the new GameString on success or an `err` if invalid.
This is far simpler than maintaining separate UHP sessions per player, but
amounts to using a weird subset of UHP-ish semantics.

# AI Player as viewer or engine?

Viewer:
  * Pros:
    * Much simpler to implement an AI
    * Correctly assumes that the server determines game state
  * Cons:
    * Many commands won't be valid in roach (e.g. undo, options, bestmove,
      newgame), which makes it feel like an invalid use of UHP
    * Requires modifications to UHP since currently engines don't make moves
      on their own, and trusting AIs to correctly prompt for `bestmove` and
      then make it is bad

Engine:
  * Pros:
    * Works w/ UHP 1.0
    * Allows use of `bestmove` command by server, setting AI options e.g. time
      controls, and `id` for displaying the AI info
    * Lets e.g. Mzinga.Viewer work as expected, allowing for easy playtesting
    * Lets server verify that each players' state is in sync
  * Cons:
    * More complicated logic must be implemented in every AI
    * No UHP way of communicating to an engine that its state is invalid, so
      desyncs would just have to be handled via forfeits
    * the 3 Engine Model is a bit nuts
>>>>>>> 54f3a8c... update UHP proposal with all kindsa models
