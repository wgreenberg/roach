Universal Hive Protocol
=======================

https://github.com/jonthysell/Mzinga/wiki/UniversalHiveProtocol

New command: `playandwait MoveString`

Instructs the engine to validate/play the given MoveString, printing a
GameString if successful, and then await a response turn from another player.
When this turn is submitted (either by the Engine's AI or by another player),
the Engine will then print that MoveString and resulting GameString, and then
finally "ok", separated by a newline.

For Example:

```
playandwait wA1
Base;InProgress;Black[1];wA1
bB1 -wA1
Base;InProgress;White[2];wA1;bB1 -wA1
ok
```

Here, white played their first Ant, and then awaited black's response. Black
then played their beetle next to white's ant, and play returned to white.
