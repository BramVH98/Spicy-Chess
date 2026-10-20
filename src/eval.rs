use crate::board::*;

const VALUE: [i32; 6] = [100, 320, 330, 500, 900, 0];

pub fn value(piece: usize) -> i32{
    VALUE[piece]
}

fn centre(sq: usize) -> i32{
    let f = (sq % 8) as i32;
    let r = (sq / 8) as i32;
    f.min(7 - f) + r.min(7 - r)
}

pub fn evaluate(b: &Board) -> i32{
    let mut score = 0;
    for color in 0..2{
        let mut s = 0;
        for piece in 0..6{
            let mut bb = b.pieces[piece] & b.color[color];
            while bb != 0{
                let sq = bb.trailing_zeros() as usize;
                bb &= bb - 1;
                s += VALUE[piece];
                s += match piece{
                    PAWN => {
                        let adv = (if color == 0 { sq / 8 } else { 7 - sq / 8 }) as i32;
                        adv * 5 + centre(sq) * 2
                    }
                    KNIGHT => centre(sq) * 6,
                    BISHOP => centre(sq) * 3,
                    QUEEN => centre(sq),
                    _ => 0,
                };
            }
        }
        if color == 0{
            score += s;
        } else{
            score -= s;
        }
    }
    if b.side == Color::White { score } else { -score }
}