use core::f32;

use crate::engine::board::{FullHistory, MctsHistory};
use crate::engine::{Board, Move, MoveList};
use rand::RngExt;
use rand::rngs::SmallRng;

type NodeIndex = usize;

#[derive(PartialEq, Clone, Copy)]
enum Solved {
    None,
    Win,
    Loss,
    Draw,
}

struct Node {
    parent: Option<NodeIndex>,
    children: Vec<NodeIndex>,
    untried_moves: MoveList,
    visits: u32,
    wins: f32,
    parent_move: Option<Move>,
    solved_value: Solved,
}

impl Node {
    fn new(
        board: &Board<MctsHistory>,
        parent: Option<NodeIndex>,
        parent_move: Option<Move>,
        solved_value: Solved,
    ) -> Self {
        let mut moves = MoveList::new();
        if !board.is_draw() {
            board.get_legal_moves_list(&mut moves);
        }
        Node {
            parent,
            children: Vec::new(),
            untried_moves: moves,
            visits: 0,
            wins: 0.0,
            parent_move,
            solved_value,
        }
    }
}

pub struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    pub fn new(board: &Board<FullHistory>) -> Self {
        let mcts_board = board.to_mcts_board();
        Tree {
            nodes: vec![Node::new(&mcts_board, None, None, Solved::None)],
        }
    }

    fn children_value_from_parent_perspective(&self, child: NodeIndex) -> f32 {
        match self.nodes[child].solved_value {
            Solved::Win => 0.0,
            Solved::Loss => 1.0,
            Solved::Draw => 0.5,
            Solved::None => 1.0 - (self.nodes[child].wins / self.nodes[child].visits as f32),
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
                self.children_value_from_parent_perspective(child_idx)
                    + c * (parent_visits.ln() / child.visits as f32).sqrt()
            };

            if uct > best_uct {
                best_uct = uct;
                best_child = Some(child_idx);
            }
        }

        best_child
    }

    fn select(&self, board: &mut Board<MctsHistory>) -> NodeIndex {
        let mut current: NodeIndex = 0; // Start at root
        while self.nodes[current].untried_moves.count == 0
            && !self.nodes[current].children.is_empty()
        {
            if self.nodes[current].solved_value != Solved::None {
                break;
            }
            if let Some(best_child_index) = self.best_uct_child(current) {
                board.make_move(self.nodes[best_child_index].parent_move.unwrap());
                current = best_child_index;
            } else {
                break;
            }
        }
        current
    }

    fn expand(
        &mut self,
        board: &mut Board<MctsHistory>,
        node: NodeIndex,
        rng: &mut SmallRng,
    ) -> NodeIndex {
        if self.nodes[node].untried_moves.count == 0
            || self.nodes[node].solved_value != Solved::None
        {
            return node;
        }
        let index = rng.random_range(0..self.nodes[node].untried_moves.count);
        if let Some(untried_move) = self.nodes[node].untried_moves.swap_pop(index) {
            board.make_move(untried_move);
            let child_idx = self.nodes.len();
            let solved_value = if board.is_draw() {
                Solved::Draw
            } else if !board.has_legal_moves() {
                if board.is_king_checked() {
                    Solved::Loss
                } else {
                    Solved::Draw
                }
            } else {
                Solved::None
            };
            self.nodes.push(Node::new(
                board,
                Some(node),
                Some(untried_move),
                solved_value,
            ));
            self.nodes[node].children.push(child_idx);
            child_idx
        } else {
            node
        }
    }

    fn rollout(
        &self,
        board: &mut Board<MctsHistory>,
        node: NodeIndex,
        move_buffer: &mut MoveList,
        rng: &mut SmallRng,
    ) -> f32 {
        match self.nodes[node].solved_value {
            Solved::None => (),
            Solved::Win => return 1.0,
            Solved::Loss => return 0.0,
            Solved::Draw => return 0.5,
        }
        let starting_player = board.side_to_move;
        while !board.is_draw() {
            board.get_legal_moves_list(move_buffer);
            let num_moves = move_buffer.count;
            if num_moves == 0 {
                if board.is_king_checked() {
                    if board.side_to_move != starting_player {
                        return 1.0; // Starting player won
                    } else {
                        return 0.0; // Starting player lost
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

    fn try_solve(&mut self, node: NodeIndex) {
        if self.nodes[node].solved_value == Solved::None {
            let mut new_solved_value = Solved::None;
            let mut all_win = true;
            let mut has_draw = false;
            let mut is_expanded = self.nodes[node].untried_moves.count == 0;
            for &child_index in &self.nodes[node].children {
                match self.nodes[child_index].solved_value {
                    Solved::Loss => {
                        new_solved_value = Solved::Win;
                        all_win = false;
                        has_draw = false;
                        break;
                    }
                    Solved::Draw => {
                        all_win = false;
                        has_draw = true;
                    }
                    Solved::None => {
                        is_expanded = false;
                    }
                    Solved::Win => (),
                }
            }
            if is_expanded {
                if all_win {
                    new_solved_value = Solved::Loss;
                } else if has_draw {
                    new_solved_value = Solved::Draw;
                }
            }
            self.nodes[node].solved_value = new_solved_value;
        }
    }

    fn backpropagate(&mut self, leaf: NodeIndex, mut value: f32) {
        let mut current: Option<NodeIndex> = Some(leaf);
        while let Some(index) = current {
            self.nodes[index].visits += 1;
            self.nodes[index].wins += value;
            self.try_solve(index);
            value = 1.0 - value;
            current = self.nodes[index].parent;
        }
    }

    fn collect_subtree(
        old_nodes: &[Node],
        old_index: NodeIndex,
        new_nodes: &mut Vec<Node>,
        parent: Option<NodeIndex>,
    ) -> NodeIndex {
        let new_index = new_nodes.len();
        let old_node = &old_nodes[old_index];
        let new_node = Node {
            parent,
            children: Vec::new(),
            untried_moves: old_node.untried_moves.clone(),
            visits: old_node.visits,
            wins: old_node.wins,
            parent_move: old_node.parent_move,
            solved_value: old_node.solved_value,
        };
        new_nodes.push(new_node);
        for &old_child_index in &old_node.children {
            let new_child_index =
                Self::collect_subtree(old_nodes, old_child_index, new_nodes, Some(new_index));
            new_nodes[new_index].children.push(new_child_index);
        }
        new_index
    }

    pub fn advance_root(&mut self, board: &Board<FullHistory>, move_made: Move) {
        let new_root_old_index = self.nodes[0]
            .children
            .iter()
            .copied()
            .find(|&c| self.nodes[c].parent_move == Some(move_made));
        if let Some(old_index) = new_root_old_index {
            let mut new_nodes = Vec::new();
            Self::collect_subtree(&self.nodes, old_index, &mut new_nodes, None);
            self.nodes = new_nodes;
        } else {
            self.nodes = vec![Node::new(&board.to_mcts_board(), None, None, Solved::None)];
        }
    }

    pub fn search(
        &mut self,
        board: &mut Board<FullHistory>,
        iteration_count: usize,
        rng: &mut SmallRng,
    ) -> Option<Move> {
        let mcts_board = board.to_mcts_board();
        let mut moves = MoveList::new();
        mcts_board.get_legal_moves_list(&mut moves);
        if board.is_draw() || moves.count == 0 {
            return None;
        }

        // Iterate over the tree
        for _ in 0..iteration_count {
            if self.nodes[0].solved_value != Solved::None {
                break;
            }
            let mut iteration_board = mcts_board.clone();
            let selected_node = self.select(&mut iteration_board);
            let new_node = self.expand(&mut iteration_board, selected_node, rng);
            let score = self.rollout(&mut iteration_board, new_node, &mut moves, rng);
            self.backpropagate(new_node, score);
        }

        match self.nodes[0].solved_value {
            Solved::Win => {
                for &child_index in &self.nodes[0].children {
                    if self.nodes[child_index].solved_value == Solved::Loss {
                        return self.nodes[child_index].parent_move;
                    }
                }
            }
            Solved::Draw => {
                for &child_index in &self.nodes[0].children {
                    if self.nodes[child_index].solved_value == Solved::Draw {
                        return self.nodes[child_index].parent_move;
                    }
                }
            }
            _ => (),
        }
        self.nodes[0]
            .children
            .iter()
            .max_by_key(|&&c| self.nodes[c].visits)
            .and_then(|&c| self.nodes[c].parent_move)
    }
}
