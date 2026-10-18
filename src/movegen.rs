use crate::attacks::*;
use crate::board::*;

#[derive(Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub promo: u8,
}

impl Move {
    pub fn new(from: usize, to: usize, promo: usize) -> Move {
        Move { from: from as u8, to: to as u8, promo: promo as u8 }
    }

    pub fn to_uci(&self) -> String {
        let sq = |s: u8| format!("{}{}", (b'a' + s % 8) as char, (b'1' + s / 8) as char);
        let mut s = format!("{}{}", sq(self.from), sq(self.to));
        match self.promo {
            1 => s.push('n'),
            2 => s.push('b'),
            3 => s.push('r'),
            4 => s.push('q'),
            _ => {}
        }
        s
    }
}

fn castle_mask(sq: usize) -> u8 {
    match sq {
        0 => !2,
        4 => !3,
        7 => !1,
        56 => !8,
        60 => !12,
        63 => !4,
        _ => 0xFF,
    }
}

fn push_pawn(moves: &mut Vec<Move>, from: usize, to: usize, promo_rank: usize) {
    if to / 8 == promo_rank {
        for p in [QUEEN, ROOK, BISHOP, KNIGHT] {
            moves.push(Move::new(from, to, p));
        }
    } else {
        moves.push(Move::new(from, to, 0));
    }
}

impl Board {
    pub fn piece_on(&self, sq: usize) -> Option<usize> {
        let bit = 1u64 << sq;
        for p in 0..6 {
            if self.pieces[p] & bit != 0 {
                return Some(p);
            }
        }
        None
    }

    pub fn attacked(&self, sq: usize, by: usize, t: &Tables) -> bool {
        let occ = self.color[0] | self.color[1];
        let c = self.color[by];
        if t.pawn[1 - by][sq] & self.pieces[PAWN] & c != 0 {
            return true;
        }
        if t.knight[sq] & self.pieces[KNIGHT] & c != 0 {
            return true;
        }
        if t.king[sq] & self.pieces[KING] & c != 0 {
            return true;
        }
        if bishop_attacks(sq, occ) & (self.pieces[BISHOP] | self.pieces[QUEEN]) & c != 0 {
            return true;
        }
        if rook_attacks(sq, occ) & (self.pieces[ROOK] | self.pieces[QUEEN]) & c != 0 {
            return true;
        }
        false
    }

    pub fn make_move(&self, m: Move) -> Board {
        let mut b = self.clone();
        let us = self.side as usize;
        let them = 1 - us;
        let from = m.from as usize;
        let to = m.to as usize;
        let piece = self.piece_on(from).unwrap();
        let from_bb = 1u64 << from;
        let to_bb = 1u64 << to;
        let mut reset_clock = piece == PAWN;

        if let Some(cap) = self.piece_on(to) {
            b.pieces[cap] &= !to_bb;
            b.color[them] &= !to_bb;
            reset_clock = true;
        }

        if piece == PAWN && Some(m.to) == self.ep && from % 8 != to % 8 {
            let cap_sq = if us == 0 { to - 8 } else { to + 8 };
            b.pieces[PAWN] &= !(1u64 << cap_sq);
            b.color[them] &= !(1u64 << cap_sq);
        }

        b.pieces[piece] &= !from_bb;
        b.color[us] &= !from_bb;
        let placed = if m.promo != 0 { m.promo as usize } else { piece };
        b.pieces[placed] |= to_bb;
        b.color[us] |= to_bb;

        if piece == KING && (to as i32 - from as i32).abs() == 2 {
            let (rf, rt) = match to {
                6 => (7, 5),
                2 => (0, 3),
                62 => (63, 61),
                _ => (56, 59),
            };
            let both = (1u64 << rf) | (1u64 << rt);
            b.pieces[ROOK] ^= both;
            b.color[us] ^= both;
        }

        b.ep = None;
        if piece == PAWN && (to as i32 - from as i32).abs() == 16 {
            b.ep = Some(((from + to) / 2) as u8);
        }

        b.castling &= castle_mask(from) & castle_mask(to);
        b.halfmove = if reset_clock { 0 } else { self.halfmove + 1 };
        if us == 1 {
            b.fullmove += 1;
        }
        b.side = if us == 0 { Color::Black } else { Color::White };
        b
    }

    pub fn pseudo_moves(&self, t: &Tables) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let us = self.side as usize;
        let own = self.color[us];
        let enemy = self.color[1 - us];
        let occ = own | enemy;

        for piece in KNIGHT..=KING {
            let mut bb = self.pieces[piece] & own;
            while bb != 0 {
                let from = bb.trailing_zeros() as usize;
                bb &= bb - 1;
                let mut targets = match piece {
                    KNIGHT => t.knight[from],
                    BISHOP => bishop_attacks(from, occ),
                    ROOK => rook_attacks(from, occ),
                    QUEEN => queen_attacks(from, occ),
                    _ => t.king[from],
                } & !own;
                while targets != 0 {
                    let to = targets.trailing_zeros() as usize;
                    targets &= targets - 1;
                    moves.push(Move::new(from, to, 0));
                }
            }
        }

        let up: i32 = if us == 0 { 8 } else { -8 };
        let start_rank = if us == 0 { 1 } else { 6 };
        let promo_rank = if us == 0 { 7 } else { 0 };
        let mut pawns = self.pieces[PAWN] & own;
        while pawns != 0 {
            let from = pawns.trailing_zeros() as usize;
            pawns &= pawns - 1;

            let one = (from as i32 + up) as usize;
            if occ & (1u64 << one) == 0 {
                push_pawn(&mut moves, from, one, promo_rank);
                if from / 8 == start_rank {
                    let two = (from as i32 + 2 * up) as usize;
                    if occ & (1u64 << two) == 0 {
                        moves.push(Move::new(from, two, 0));
                    }
                }
            }

            let mut caps = t.pawn[us][from] & enemy;
            while caps != 0 {
                let to = caps.trailing_zeros() as usize;
                caps &= caps - 1;
                push_pawn(&mut moves, from, to, promo_rank);
            }

            if let Some(ep) = self.ep {
                if t.pawn[us][from] & (1u64 << ep) != 0 {
                    moves.push(Move::new(from, ep as usize, 0));
                }
            }
        }

        if us == 0 {
            if self.castling & 1 != 0
                && occ & ((1u64 << 5) | (1u64 << 6)) == 0
                && !self.attacked(4, 1, t)
                && !self.attacked(5, 1, t)
            {
                moves.push(Move::new(4, 6, 0));
            }
            if self.castling & 2 != 0
                && occ & ((1u64 << 1) | (1u64 << 2) | (1u64 << 3)) == 0
                && !self.attacked(4, 1, t)
                && !self.attacked(3, 1, t)
            {
                moves.push(Move::new(4, 2, 0));
            }
        } else {
            if self.castling & 4 != 0
                && occ & ((1u64 << 61) | (1u64 << 62)) == 0
                && !self.attacked(60, 0, t)
                && !self.attacked(61, 0, t)
            {
                moves.push(Move::new(60, 62, 0));
            }
            if self.castling & 8 != 0
                && occ & ((1u64 << 57) | (1u64 << 58) | (1u64 << 59)) == 0
                && !self.attacked(60, 0, t)
                && !self.attacked(59, 0, t)
            {
                moves.push(Move::new(60, 58, 0));
            }
        }

        moves
    }

    pub fn legal_moves(&self, t: &Tables) -> Vec<Move> {
        let us = self.side as usize;
        self.pseudo_moves(t)
            .into_iter()
            .filter(|&m| {
                let nb = self.make_move(m);
                let ksq = (nb.pieces[KING] & nb.color[us]).trailing_zeros() as usize;
                !nb.attacked(ksq, 1 - us, t)
            })
            .collect()
    }
}

pub fn perft(b: &Board, depth: u32, t: &Tables) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = b.legal_moves(t);
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut n = 0;
    for m in moves {
        n += perft(&b.make_move(m), depth - 1, t);
    }
    n
}