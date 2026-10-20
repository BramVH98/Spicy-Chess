use crate::attacks::Tables;
use crate::board::*;
use crate::eval::*;
use crate::movegen::*;
use std::time::Instant;

pub const MATE: i32 = 30000;
const INF: i32 = 32000;

fn is_capture(b: &Board, m: Move) -> bool {
    b.piece_on(m.to as usize).is_some()
        || (b.piece_on(m.from as usize) == Some(PAWN) && m.from % 8 != m.to % 8)
}

// Higher = try first. Captures of big pieces by small pieces go first.
fn order_score(b: &Board, m: Move) -> i32 {
    let mut s = 0;
    let attacker = b.piece_on(m.from as usize).unwrap_or(0);
    if let Some(victim) = b.piece_on(m.to as usize) {
        s += 10000 + value(victim) * 10 - value(attacker) / 10;
    } else if is_capture(b, m) {
        s += 10000 + 1000 - value(attacker) / 10; // en passant
    }
    if m.promo != 0 {
        s += 9000 + m.promo as i32;
    }
    s
}

fn king_attacked(b: &Board, color: usize, t: &Tables) -> bool {
    let k = b.pieces[KING] & b.color[color];
    k != 0 && b.attacked(k.trailing_zeros() as usize, 1 - color, t)
}

fn score_str(s: i32) -> String {
    if s.abs() > MATE - 100 {
        let moves = (MATE - s.abs() + 1) / 2;
        format!("mate {}", if s > 0 { moves } else { -moves })
    } else {
        format!("cp {}", s)
    }
}

pub struct Searcher {
    pub nodes: u64,
    start: Instant,
    limit_ms: u128,
    stop: bool,
    depth_done: i32,
}

impl Searcher {
    pub fn new() -> Searcher {
        Searcher { nodes: 0, start: Instant::now(), limit_ms: 1000, stop: false, depth_done: 0 }
    }

    // Only stops once depth 1 is finished, so we always have a move
    fn check_time(&mut self) {
        if self.depth_done >= 1
            && self.nodes & 2047 == 0
            && self.start.elapsed().as_millis() >= self.limit_ms
        {
            self.stop = true;
        }
    }

    // Keeps searching captures until the position is quiet,
    // so we never stop in the middle of a trade
    fn quiesce(&mut self, b: &Board, ply: i32, mut alpha: i32, beta: i32, t: &Tables) -> i32 {
        self.nodes += 1;
        self.check_time();
        if self.stop {
            return 0;
        }
        let stand = evaluate(b);
        if ply >= 64 || stand >= beta {
            return stand;
        }
        if stand > alpha {
            alpha = stand;
        }
        let mut best = stand;
        let us = b.side as usize;
        let mut moves: Vec<Move> = b
            .pseudo_moves(t)
            .into_iter()
            .filter(|&m| is_capture(b, m) || m.promo != 0)
            .collect();
        moves.sort_by_key(|&m| -order_score(b, m));
        for m in moves {
            let nb = b.make_move(m);
            if king_attacked(&nb, us, t) {
                continue;
            }
            let score = -self.quiesce(&nb, ply + 1, -beta, -alpha, t);
            if self.stop {
                return 0;
            }
            if score > best {
                best = score;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }
        best
    }

    fn negamax(&mut self, b: &Board, mut depth: i32, ply: i32, mut alpha: i32, beta: i32, t: &Tables) -> i32 {
        self.nodes += 1;
        self.check_time();
        if self.stop {
            return 0;
        }
        if b.halfmove >= 100 {
            return 0; // 50-move rule
        }
        if ply >= 64 {
            return evaluate(b);
        }
        let us = b.side as usize;
        let in_check = king_attacked(b, us, t);
        if in_check {
            depth += 1; // look deeper when in check
        }
        if depth <= 0 {
            return self.quiesce(b, ply, alpha, beta, t);
        }

        let mut moves = b.pseudo_moves(t);
        moves.sort_by_key(|&m| -order_score(b, m));

        let mut best = -INF;
        let mut legal = 0;
        for m in moves {
            let nb = b.make_move(m);
            if king_attacked(&nb, us, t) {
                continue;
            }
            legal += 1;
            let score = -self.negamax(&nb, depth - 1, ply + 1, -beta, -alpha, t);
            if self.stop {
                return 0;
            }
            if score > best {
                best = score;
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }
        if legal == 0 {
            return if in_check { -MATE + ply } else { 0 };
        }
        best
    }

    // Iterative deepening: search depth 1, then 2, then 3...
    // Each round gets better move ordering from the previous one.
    pub fn think(&mut self, b: &Board, max_depth: i32, limit_ms: u128, t: &Tables) -> Option<Move> {
        self.start = Instant::now();
        self.limit_ms = limit_ms;
        self.stop = false;
        self.nodes = 0;
        self.depth_done = 0;

        let mut root = b.legal_moves(t);
        if root.is_empty() {
            return None;
        }
        root.sort_by_key(|&m| -order_score(b, m));
        let mut best = root[0];

        for depth in 1..=max_depth {
            let mut alpha = -INF;
            let mut best_this = root[0];
            let mut best_score = -INF;
            for &m in &root {
                let nb = b.make_move(m);
                let score = -self.negamax(&nb, depth - 1, 1, -INF, -alpha, t);
                if self.stop {
                    break;
                }
                if score > best_score {
                    best_score = score;
                    best_this = m;
                }
                if score > alpha {
                    alpha = score;
                }
            }
            if self.stop {
                break; // throw away the unfinished round
            }
            best = best_this;
            self.depth_done = depth;
            let ms = self.start.elapsed().as_millis();
            println!(
                "info depth {} score {} nodes {} time {} pv {}",
                depth, score_str(best_score), self.nodes, ms, best.to_uci()
            );
            if let Some(pos) = root
                .iter()
                .position(|x| x.from == best.from && x.to == best.to && x.promo == best.promo)
            {
                root.remove(pos);
                root.insert(0, best);
            }
            if best_score.abs() > MATE - 100 || ms * 2 > self.limit_ms {
                break;
            }
        }
        Some(best)
    }
}