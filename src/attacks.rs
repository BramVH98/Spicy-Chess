fn step_attacks(sq: usize, deltas: &[(i32, i32)]) -> u64{
    let f = (sq % 8) as i32;
    let r = (sq / 8) as i32;
    let mut bb = 0u64;
    for &(df, dr) in deltas{
        let nf = f + df;
        let nr = r + dr;
        if nf >= 0 && nf < 8 && nr >= 0 && nr < 8{
            bb |= 1u64 << (nr * 8 + nf)
        }
    }
    bb
}

pub struct Tables{
    pub knight: [u64; 64],
    pub king: [u64; 64],
    pub pawn: [[u64; 64]; 2],
}

impl Tables{
    pub fn new() -> Tables{
        let knight_d = [
            (1, 2), (2, 1), (2, -1), (1, -2),
            (-1, -2), (-2, -1), (-2, 1), (-1, 2),
        ];
        let king_d = [
            (1, 0), (1, 1), (0, 1), (-1, 1),
            (-1, 0), (-1, -1), (0, -1), (1, -1),
        ];
        let mut t = Tables{
            knight: [0; 64],
            king: [0; 64],
            pawn: [[0; 64]; 2],
        };
        for sq in 0..64{
            t.knight[sq] = step_attacks(sq, &knight_d);
            t.king[sq] = step_attacks(sq, &king_d);
            t.pawn[0][sq] = step_attacks(sq, &[(-1, 1), (1, 1)]);
            t.pawn[1][sq] = step_attacks(sq, &[(-1, -1), (1, -1)]);
        }
        t
    }
}

fn slide (sq: usize, occ: u64, dirs: &[(i32, i32)]) -> u64{
    let mut bb = 0u64;
    for &(df, dr) in dirs{
        let mut f = (sq % 8) as i32 + df;
        let mut r = (sq / 8) as i32 + dr;
        while f >= 0 && f < 8 && r >= 0 && r < 8{
            let bit = 1u64 << (r * 8 + f);
            bb |= bit;
            if occ & bit != 0{
                break;
            }
            f += df;
            r += dr;
        }
    }
    bb
}

pub fn rook_attacks(sq: usize, occ: u64) -> u64{
    slide(sq, occ, &[(1, 0), (-1, 0), (0, 1), (0, -1)])
}

pub fn bishop_attacks(sq: usize, occ: u64) -> u64{
    slide(sq, occ, &[(1, 1), (1, -1), (-1, 1), (-1, -1)])
}

pub fn queen_attacks(sq: usize, occ: u64) -> u64{
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}

pub  fn print_bb(bb: u64){
    for rank in (0..8).rev(){
        let mut line = format!("{} ", rank + 1);
        for file in 0..8{
            let bit = 1u64 << (rank * 8 + file);
            line.push(if bb & bit != 0 { 'X' } else { '.' });
            line.push(' ');
        }
        println!("{}", line);
    }
    println!("  a b c d e f g h\n");
}