use crate::engine::{Board, Move, MoveList};
use rand::RngExt;
use rand::rngs::SmallRng;

pub fn get_random_move(board: &mut Board, rng: &mut SmallRng) -> Option<Move> {
    let mut moves = MoveList::new();
    board.get_legal_moves_list(&mut moves);
    if moves.count == 0 || board.is_draw() {
        return None;
    };
    let index = rng.random_range(0..moves.count);
    moves.get(index)
}

type NodeIndex = usize;

struct Node {
    parent: Option<NodeIndex>,
    children: Vec<NodeIndex>,
    untried_moves: MoveList,
    visits: u32,
    wins: f32,
    move_from_parent: Option<Move>,
}

impl Node {
    fn new(board: &Board, parent: NodeIndex, parent_move: Move) -> Self {
        let mut moves = MoveList::new();
        if !board.is_draw() {
            board.get_legal_moves_list(&mut moves);
        }
        Node {
            parent: Some(parent),
            children: Vec::new(),
            untried_moves: moves,
            visits: 0,
            wins: 0.0,
            move_from_parent: Some(parent_move),
        }
    }
}

struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    fn new(board: &mut Board) -> Self {
        let mut moves = MoveList::new();
        board.get_legal_moves_list(&mut moves);
        Tree {
            nodes: vec![Node {
                parent: None,
                children: Vec::new(),
                untried_moves: moves,
                visits: 0,
                wins: 0.0,
                move_from_parent: None,
            }],
        }
    }

    fn best_uct_child(&self, parent_index: NodeIndex) -> Option<NodeIndex> {
        let parent = &self.nodes[parent_index];
        let parent_visits = parent.visits as f32;

        let mut best_child = parent.children.first().copied();
        let mut best_uct = f32::NEG_INFINITY;
        let c = 1.41;

        for &child_idx in &parent.children {
            let child = &self.nodes[child_idx];
            let uct = if child.visits == 0 {
                f32::INFINITY
            } else {
                (child.wins / child.visits as f32)
                    + c * (parent_visits.ln() / child.visits as f32).sqrt()
            };

            if uct > best_uct {
                best_uct = uct;
                best_child = Some(child_idx);
            }
        }

        best_child
    }

    fn select(&self, board: &mut Board) -> NodeIndex {
        let mut current: NodeIndex = 0; // Start at root
        while self.nodes[current].untried_moves.count == 0
            && !self.nodes[current].children.is_empty()
        {
            if let Some(best_child_index) = self.best_uct_child(current) {
                board.make_move(self.nodes[best_child_index].move_from_parent.unwrap());
                current = best_child_index;
            } else {
                break;
            }
        }
        current
    }

    fn expand(&mut self, board: &mut Board, node: NodeIndex, rng: &mut SmallRng) -> NodeIndex {
        if self.nodes[node].untried_moves.count == 0 {
            return node;
        }
        let index = rng.random_range(0..self.nodes[node].untried_moves.count);
        if let Some(untried_move) = self.nodes[node].untried_moves.swap_pop(index) {
            board.make_move(untried_move);
            let child_idx = self.nodes.len();
            self.nodes.push(Node::new(board, node, untried_move));
            self.nodes[node].children.push(child_idx);
            child_idx
        } else {
            node
        }
    }

    fn rollout(&self, board: &mut Board, move_buffer: &mut MoveList, rng: &mut SmallRng) -> f32 {
        let starting_player = board.side_to_move;
        while !board.is_draw() {
            board.get_legal_moves_list(move_buffer);
            let num_moves = move_buffer.count;
            if num_moves == 0 {
                if board.is_king_checked() {
                    if board.side_to_move != starting_player {
                        return 0.0; // Checkmate loss
                    } else {
                        return 1.0; // Checkmate win
                    }
                } else {
                    break;
                }
            }
            let mv = move_buffer.get(rng.random_range(0..num_moves)).unwrap();
            board.make_move(mv);
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

pub fn mcts_search(board: &mut Board, iteration_number: usize, rng: &mut SmallRng) -> Option<Move> {
    let mut moves = MoveList::new();
    board.get_legal_moves_list(&mut moves);
    if board.is_draw() || moves.count == 0 {
        return None;
    }

    // Create tree
    let mut tree = Tree::new(board);
    let mut iteration_board;
    // Iterate over the tree
    for _ in 0..iteration_number {
        iteration_board = board.clone();
        let selected_node = tree.select(&mut iteration_board);
        let new_node = tree.expand(&mut iteration_board, selected_node, rng);
        let score = tree.rollout(&mut iteration_board, &mut moves, rng);
        tree.backpropagate(new_node, score);
    }

    tree.nodes[0]
        .children
        .iter()
        .max_by_key(|&&c| tree.nodes[c].visits)
        .and_then(|&c| tree.nodes[c].move_from_parent)
}
