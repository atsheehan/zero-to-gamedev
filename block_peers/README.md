# Block Peers

The goal of this project is to write a networked Tetris-clone in Rust.

## Iteration 1 - Basic Mechanics

This deliverable defines the basic mechanics of controlling the active piece and attaching bricks to the grid.

* The player has an active piece that they can move around the grid.
* They can move the piece left, right, or down by one cell, rotate the piece clockwise, or drop the piece to the ground.
* The piece drops by one cell every 2 seconds.
* If the piece collides with the bottom of the grid or an occupied cell, the bricks from the piece are copied to the cells.
* After attaching a piece to the grid, any complete lines are cleared out and bricks above the line are dropped down.
* After attaching a piece to the grid, a new active piece is generated and placed at the top of the grid.
* The generated piece can be one of seven possible shapes, chosen at random.
* If a new active piece is generated on top of an occupied cell, the game is over.

### Story Breakdown

* [x] Display a grid that is 20 cells high and 10 cells wide.
* [x] Display a T-shaped piece at the top of the grid.
* [x] The player can move the piece left, right, or down 1 cell with the A, D, and S keys, respectively. The piece cannot move past the grid wall.
* [x] When the active piece collides with the bottom of the grid, it attaches to the grid (i.e. the player can no longer control it).
* [x] After attaching to the grid, a new T-shaped piece is generated at the top of the grid.
* [x] When the active piece collides with an occupied cell on the grid, it attaches to the grid.
* [x] The player can drop the piece to the bottom of the grid by pressing W.
* [x] If the player has not moved the active piece down within 2 seconds, the piece automatically moves down one cell. If it collides with the floor or an occupied cell, it is attached to the grid.
* [x] The player can rotate the piece clockwise 90 degrees by pressing E.
* [x] Proper collision detection on rotated pieces
* [x] When a piece hits the ground it maintains its rotation
* [x] When generating a new piece, any of the seven possible shapes can be chosen.
* [ ] Add a 'center' functionality to a piece so it can be moved to correct starting point
* [ ] After a piece is attached to the grid, any full lines are cleared out and bricks above the line are dropped down.
* [ ] If a new piece is generated on top of an occupied cell, the game is over.

### Discuss Next Pairing Session

* Border being set to white for collision detection
* Global vs. local iterator and fixing the crashes

## Backlog

* Track lines cleared / points
* Render textures
* Adding a second player / competitive play
* Sounds / music
* Title screen
* Networked clients

## Glossary

When writing Tetris clones in the past, having a consistent name for everything makes the process a lot easier.

* A **grid** defines the main playing field where the player can move and rotate shapes.
* A **cell** refers to any (row, col) coordinate in the grid. It can be empty or occupied by a brick.
* A **brick** occupies a cell. It can be attached to the grid or part of the active piece moving around.
* A **piece** is the collection of bricks a player controls. The arrangement of bricks is defined by the **shape** of the piece.
