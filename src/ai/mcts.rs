use crate::engine::{Board, GameStruct, Move, MoveList};
use rand::RngExt;
use rand::rngs::SmallRng;

#[derive(Clone, Copy)]
struct MctsGame {
    b: Board,
    hash_history: [u64; 256],
}

impl MctsGame {
    fn from_fullgame(fullgame: &GameStruct) -> Self {
        let mut hashes = [0; 256];
        let hmc = fullgame.b.halfmove_clock as usize;
        hashes[hmc] = fullgame.b.hash;
        let mut copy = fullgame.clone();
        for i in 0..hmc {
            copy.unmake_move();
            hashes[hmc - i] = copy.b.hash;
        }
        Self {
            b: fullgame.b,
            hash_history: hashes,
        }
    }

    fn make_move(&mut self, mv: Move) {
        self.hash_history[self.b.halfmove_clock as usize] = self.b.make_move(mv).hash;
    }

    fn is_threefold_repetition(&self) -> bool {
        self.hash_history[..self.b.halfmove_clock as usize]
            .iter()
            .rev()
            .skip(1)
            .step_by(2)
            .filter(|&&h| h == self.b.hash)
            .take(2)
            .count()
            >= 2
    }

    fn is_draw(&self) -> bool {
        self.b.is_draw() || self.is_threefold_repetition()
    }
}

type NodeIndex = usize;

struct Node {
    parent: Option<NodeIndex>,
    children: Vec<NodeIndex>,
    untried_moves: MoveList,
    visits: u32,
    wins: f32,
    parent_move: Option<Move>,
}

impl Node {
    fn new(game: &MctsGame, parent: Option<NodeIndex>, parent_move: Option<Move>) -> Self {
        let mut moves = MoveList::default();
        if !game.is_draw() {
            game.b.get_legal_moves_list(&mut moves);
        }
        Self {
            parent,
            children: Vec::new(),
            untried_moves: moves,
            visits: 0,
            wins: 0.0,
            parent_move,
        }
    }
}

struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    fn new(game: &MctsGame) -> Self {
        Self {
            nodes: vec![Node::new(game, None, None)],
        }
    }
    fn best_uct_child(&self, parent_index: NodeIndex) -> Option<NodeIndex> {
        let parent = &self.nodes[parent_index];
        #[allow(clippy::cast_precision_loss)]
        let parent_visits = parent.visits as f32;

        let mut best_child = parent.children.first().copied();
        let mut best_uct = f32::NEG_INFINITY;
        let c = 1.41;

        for &child_idx in &parent.children {
            let child = &self.nodes[child_idx];
            let uct = (1.0 - child.wins / child.visits as f32)
                + c * (parent_visits.ln() / child.visits as f32).sqrt();

            if uct > best_uct {
                best_uct = uct;
                best_child = Some(child_idx);
            }
        }

        best_child
    }

    fn select(&self, game: &mut MctsGame) -> NodeIndex {
        let mut current: NodeIndex = 0; // Start at root
        while self.nodes[current].untried_moves.count == 0
            && !self.nodes[current].children.is_empty()
        {
            if let Some(best_child_index) = self.best_uct_child(current) {
                game.make_move(self.nodes[best_child_index].parent_move.unwrap());
                current = best_child_index;
            } else {
                break;
            }
        }
        current
    }

    fn expand(&mut self, game: &mut MctsGame, node: NodeIndex, rng: &mut SmallRng) -> NodeIndex {
        if self.nodes[node].untried_moves.count == 0 {
            return node;
        }
        let index = rng.random_range(0..self.nodes[node].untried_moves.count);
        if let Some(untried_move) = self.nodes[node].untried_moves.swap_pop(index) {
            game.make_move(untried_move);
            let child_idx = self.nodes.len();
            self.nodes
                .push(Node::new(game, Some(node), Some(untried_move)));
            self.nodes[node].children.push(child_idx);
            child_idx
        } else {
            node
        }
    }

    fn rollout(game: &mut MctsGame, move_buffer: &mut MoveList, rng: &mut SmallRng) -> f32 {
        let starting_player = game.b.side_to_move;
        while !game.is_draw() {
            game.b.get_legal_moves_list(move_buffer);
            let num_moves = move_buffer.count;
            if num_moves == 0 {
                if game.b.is_king_checked() {
                    if game.b.side_to_move == starting_player {
                        return 0.0; // Checkmate loss
                    }
                    return 1.0; // Checkmate win
                }
                break;
            }
            let mv = move_buffer.get(rng.random_range(0..num_moves)).unwrap();
            game.make_move(mv);
        }
        0.5 // Draw
    }

    fn backpropagate(&mut self, leaf: NodeIndex, mut value: f32) {
        let mut current: Option<NodeIndex> = Some(leaf);
        while let Some(index) = current {
            let node = &mut self.nodes[index];
            node.visits += 1;
            node.wins += value;
            value = 1.0 - value;
            current = node.parent;
        }
    }
}

pub fn mcts_search(
    fullgame: &GameStruct,
    iteration_number: usize,
    rng: &mut SmallRng,
) -> Option<Move> {
    let mcts_board = MctsGame::from_fullgame(fullgame);
    let mut moves = MoveList::default();
    mcts_board.b.get_legal_moves_list(&mut moves);
    if fullgame.is_draw() || moves.count == 0 {
        return None;
    }

    // Create tree
    let mut tree = Tree::new(&mcts_board);
    // Iterate over the tree
    for _ in 0..iteration_number {
        let mut iteration_board = mcts_board;
        let selected_node = tree.select(&mut iteration_board);
        let new_node = tree.expand(&mut iteration_board, selected_node, rng);
        let score = Tree::rollout(&mut iteration_board, &mut moves, rng);
        tree.backpropagate(new_node, score);
    }

    tree.nodes[0]
        .children
        .iter()
        .max_by_key(|&&c| tree.nodes[c].visits)
        .and_then(|&c| tree.nodes[c].parent_move)
}
