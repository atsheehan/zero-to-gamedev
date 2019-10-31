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
* [x] After a piece is attached to the grid, any full lines are cleared out and bricks above the line are dropped down.
* [x] Add a 'center' functionality to a piece so it can be moved to correct starting point
* [x] If a new piece is generated on top of an occupied cell, the game is over.

## Iteration 2 - Networked Client

This deliverable will separate the game into client and server applications that communicate over the network. The server will run the game logic and the client will handle rendering and user input. The client and server will communicate using UDP.

This will still be a single player game, but I think splitting client and server functionality first will make it easier to add networked multiplayer functionality on top of it. This will be a barebones protocol that doesn't attempt to be secure or robust in any way, it's just to get something working.

* When the server start it listens for client connections.
* As soon as a client joins, the server starts the game and sends the initial state.
* The client renders the state and sends any user commands to the server.
* Whenever the state changes, the server sends the new state to the client.
* When the client quits, it disconnects from the server.
* The server can only have active client at a time. Other clients attempting to connect will receive a rejection message.

### Story Breakdown

* [x] Server process listens on UDP port 4485. When a client connects, it responds with an ACK message.
* [x] Server sends the initial game state to the client and the client renders it.
* [x] Client sends any user commands to the server and the server responds with the new state.
* [x] Server sends game state whenever it changes.
* [ ] Client disconnects before quitting.
* [ ] Server rejects connections while a session is active.

## Iteration 3 - Multiplayer

This deliverable will support multiple players in a single game. A game server listens for connections from clients, and once two clients have connected it will start the game. Both players have their own grid and the game continues until either player loses.

### Story Breakdown

TODO

## Backlog / Cleanup

### Gameplay

* [ ] As time goes on the game speeds up and gets more challenging
* [ ] Implement AI for multiplayer
* [ ] Clearing multiple lines affects other players grid somehow in multiplayer (e.g. drops random pieces)
* [ ] Implement the [Super Rotation System (SRS)](https://tetris.wiki/Super_Rotation_System) standard for rotating pieces
* [ ] Prevent instant moves (i.e. have a cooldown between moves to prevent an AI for instantly winning the game)
* [ ] Show the next piece
* [ ] Track points
* [ ] Add gamepad support

### Network

* [ ] More efficient grid serialization over network
* [ ] Only send network updates when there are changes
* [ ] Send partial updates over the network
* [ ] Use a shared secret to verify network messages are authentic
* [ ] Add a handshake to establish connection to ensure packet is coming from legitimate host
* [ ] Server disconnects inactive players

### Assets / Presentation

* [*] Title screen
* [ ] Sounds / music
* [ ] Add font rendering
* [ ] Add fullscreen support

## Glossary

When writing Tetris clones in the past, having a consistent name for everything makes the process a lot easier.

* A **grid** defines the main playing field where the player can move and rotate shapes.
* A **grid_cell** refers to any (row, col) coordinate in the grid. It can be empty or occupied by a brick.
* A **brick** occupies a cell. It can be attached to the grid or part of the active piece moving around.
* A **piece** is the collection of bricks a player controls. The arrangement of bricks is defined by the **shape** of the piece.

## Bugs

* Sometimes when clearing multiple lines at once blocks above the multiple lines don't properly fall down to where they're supposed to.
