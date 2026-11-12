# General info and goal of the project

This repo includes a chess engine (move generation, draw detection, make/unmake, terminal interface) as well as 2 AI architectures to play the game (Negamax & Monte Carlo Tree Search)

My goal is to make a fast and correct engine, to later allow me to concentrate on making strong game-playing AIs

# Code structure

## Cargo.toml
### Compile flags
In release mode, the code is compiled with `codegen-units = 1` and `lto = fat` to be as optimized as possible
### Dependencies
`rand` for RNG needed in random-move agent, and MCTS
`crosstem` for terminal colors and input handling

## Source code
The code is divided into 3 modules

### Engine
This module is the main engine and move generation code, it is divided into mutiple files for code cleanliness.
#### Types
Enums, structs, and constants used in later files, including:
- Piece, enum representing the chess pieces
- Color, enum for both player colors
- Move, struct to store and incode chess moves
- Square, helper struct to represent a square on the board
- MoveList, struct to store Moves efficiently
- Bitboard, helper struct to store board and piece info as bitboards and do operations on them
#### Zobrist
Zobrist struct and initializer, needed to compute initial and incremental hashes of chess positions
#### Board
Board struct and the main methods to modify it, namely:
- Make/Unmake to make and unmake moves
- Undo, struct to store the info needed to unmake a move
- From_fen, FEN parser
- Perft, function to count the number of moves at a certain depth for the current position
- 50-move and insufficient material draw detection
#### Game
GameStruct struct and methods, to store board history allowing for easier unmaking interface and threefold repetition detection
#### Bitboards
Constants needed for move generation including :
- File/Rank bitboards
- Black/White pawn attack tables
- Knight attack tables
- Rook/Bishop move tables (compute with PEXT instructions if compiled for BMI2, otherwise using magic bitboards)
- King attack tables
- Between table, stores masks of the squares between two given squares
#### Move_gen
The move generation code, using direct legal move generation (as opposed to pseudo-legal move generation with filtering), a visitor pattern (moves are fed to a given function, allowing for exploration without storing the moves).


### Bin
There are currently 2 binaries :
- Perft, the classical test for chess engines, which takes the FEN and depth as arguments
- Play, the program that allows you to match different opponents (including user input) and make AIs play multiple games to collect statistics

### AI
The Agents are:
- MCTS, an agent based on randomness and statistics, for which my goal is to make it as strong as possible without using heuristics
- Negamax, the classical basic chess AI algorithm, with alpha-beta prunning and hashtables, uses an evaluation function (material evalutation)
