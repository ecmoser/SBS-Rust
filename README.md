# SBS-Rust
A rust version of my project that solves the NYT Spelling Bee game. Original project is [here](https://github.com/ecmoser/spelling-bee-solver) and has more information about inspiration and how it works.

## Prerequisites
[Rust](https://www.rust-lang.org/tools/install) OR [Docker](https://docs.docker.com/get-docker/)

### Running Locally with Rust
1. Clone this git repository
2. Run `cargo run --release` in the root directory to start the program

### Running with Docker
1. Clone the git repo
2. Run `docker build -t sbs-rust .` to start the container
3. Run the program using `docker run -it sbs-rust` to run the program in interactive mode

## Using the Solver
Once the solver is running, it will automatically download the necessary word lists, and give you the option to solve or quit. To solve a puzzle, select solve and enter the 7-letter puzzle with the center letter capital and the rest lowercase. The solver will then output the solution to the puzzle.