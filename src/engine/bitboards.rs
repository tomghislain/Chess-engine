pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_7: u64 = 0x00FF000000000000;

const fn gen_white_pawn_attacks() -> [u64; 64] {
    let mut table = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        let sq_mask = 1u64 << sq;
        let mut mask = 0u64;
        if sq < 56 {
            mask |= (sq_mask << 7) & !FILE_H;
            mask |= (sq_mask << 9) & !FILE_A;
        }
        table[sq] = mask;
        sq += 1;
    }
    table
}

pub const WHITE_PAWN_ATTACKS: [u64; 64] = gen_white_pawn_attacks();

const fn gen_black_pawn_attacks() -> [u64; 64] {
    let mut table = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        let sq_mask = 1u64 << sq;
        let mut mask = 0u64;
        if sq > 7 {
            mask |= (sq_mask >> 7) & !FILE_A;
            mask |= (sq_mask >> 9) & !FILE_H;
        }
        table[sq] = mask;
        sq += 1;
    }
    table
}

pub const BLACK_PAWN_ATTACKS: [u64; 64] = gen_black_pawn_attacks();

const fn gen_knight_attacks() -> [u64; 64] {
    let mut table = [0u64; 64];
    let mut sq = 0;

    while sq < 64 {
        let mask = 1u64 << sq;

        let attacks = ((mask << 17) & !FILE_A)
            | ((mask << 15) & !FILE_H)
            | ((mask << 10) & !FILE_A & !FILE_B)
            | ((mask << 6) & !FILE_H & !FILE_G)
            | ((mask >> 17) & !FILE_H)
            | ((mask >> 15) & !FILE_A)
            | ((mask >> 10) & !FILE_H & !FILE_G)
            | ((mask >> 6) & !FILE_A & !FILE_B);

        table[sq as usize] = attacks;
        sq += 1;
    }

    table
}

pub const KNIGHT_ATTACKS: [u64; 64] = gen_knight_attacks();

const fn gen_rook_masks() -> [u64; 64] {
    let mut masks = [0; 64];
    let mut sq: usize = 0;
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut mask = 0;
        // North
        let mut r = rank + 1;
        while r < 7 {
            mask |= 1u64 << (r * 8 + file);
            r += 1;
        }
        // South
        let mut r = rank as i32 - 1;
        while r > 0 {
            mask |= 1u64 << (r * 8 + file as i32);
            r -= 1;
        }
        // East
        let mut f = file + 1;
        while f < 7 {
            mask |= 1u64 << (rank * 8 + f);
            f += 1;
        }
        // West
        let mut f = file as i32 - 1;
        while f > 0 {
            mask |= 1u64 << (rank as i32 * 8 + f);
            f -= 1;
        }
        masks[sq] = mask;
        sq += 1;
    }
    masks
}

const fn gen_bishop_masks() -> [u64; 64] {
    let mut masks = [0; 64];
    let mut sq: usize = 0;
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut mask = 0;
        // North-East
        let mut r = rank + 1;
        let mut f = file + 1;
        while r < 7 && f < 7 {
            mask |= 1u64 << (r * 8 + f);
            r += 1;
            f += 1;
        }
        // North-West
        let mut r = rank + 1;
        let mut f = file as i32 - 1;
        while r < 7 && f > 0 {
            mask |= 1u64 << (r as i32 * 8 + f);
            r += 1;
            f -= 1;
        }
        // South-East
        let mut r = rank as i32 - 1;
        let mut f = file + 1;
        while r > 0 && f < 7 {
            mask |= 1u64 << (r * 8 + f as i32);
            r -= 1;
            f += 1;
        }
        // South-West
        let mut r = rank as i32 - 1;
        let mut f = file as i32 - 1;
        while r > 0 && f > 0 {
            mask |= 1u64 << (r as u32 * 8 + f as u32);
            r -= 1;
            f -= 1;
        }
        masks[sq] = mask;
        sq += 1;
    }
    masks
}

pub const ROOK_MASKS: [u64; 64] = gen_rook_masks();

pub const BISHOP_MASKS: [u64; 64] = gen_bishop_masks();

// Magics and shift from https://github.com/SebLague/Chess-Coding-Adventure/
pub const ROOK_MAGIC_NUMBERS: [u64; 64] = [
    468374916371625120,
    18428729537625841661,
    2531023729696186408,
    6093370314119450896,
    13830552789156493815,
    16134110446239088507,
    12677615322350354425,
    5404321144167858432,
    2111097758984580,
    18428720740584907710,
    17293734603602787839,
    4938760079889530922,
    7699325603589095390,
    9078693890218258431,
    578149610753690728,
    9496543503900033792,
    1155209038552629657,
    9224076274589515780,
    1835781998207181184,
    509120063316431138,
    16634043024132535807,
    18446673631917146111,
    9623686630121410312,
    4648737361302392899,
    738591182849868645,
    1732936432546219272,
    2400543327507449856,
    5188164365601475096,
    10414575345181196316,
    1162492212166789136,
    9396848738060210946,
    622413200109881612,
    7998357718131801918,
    7719627227008073923,
    16181433497662382080,
    18441958655457754079,
    1267153596645440,
    18446726464209379263,
    1214021438038606600,
    4650128814733526084,
    9656144899867951104,
    18444421868610287615,
    3695311799139303489,
    10597006226145476632,
    18436046904206950398,
    18446726472933277663,
    3458977943764860944,
    39125045590687766,
    9227453435446560384,
    6476955465732358656,
    1270314852531077632,
    2882448553461416064,
    11547238928203796481,
    1856618300822323264,
    2573991788166144,
    4936544992551831040,
    13690941749405253631,
    15852669863439351807,
    18302628748190527413,
    12682135449552027479,
    13830554446930287982,
    18302628782487371519,
    7924083509981736956,
    4734295326018586370,
];
pub const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
    16509839532542417919,
    14391803910955204223,
    1848771770702627364,
    347925068195328958,
    5189277761285652493,
    3750937732777063343,
    18429848470517967340,
    17870072066711748607,
    16715520087474960373,
    2459353627279607168,
    7061705824611107232,
    8089129053103260512,
    7414579821471224013,
    9520647030890121554,
    17142940634164625405,
    9187037984654475102,
    4933695867036173873,
    3035992416931960321,
    15052160563071165696,
    5876081268917084809,
    1153484746652717320,
    6365855841584713735,
    2463646859659644933,
    1453259901463176960,
    9808859429721908488,
    2829141021535244552,
    576619101540319252,
    5804014844877275314,
    4774660099383771136,
    328785038479458864,
    2360590652863023124,
    569550314443282,
    17563974527758635567,
    11698101887533589556,
    5764964460729992192,
    6953579832080335136,
    1318441160687747328,
    8090717009753444376,
    16751172641200572929,
    5558033503209157252,
    17100156536247493656,
    7899286223048400564,
    4845135427956654145,
    2368485888099072,
    2399033289953272320,
    6976678428284034058,
    3134241565013966284,
    8661609558376259840,
    17275805361393991679,
    15391050065516657151,
    11529206229534274423,
    9876416274250600448,
    16432792402597134585,
    11975705497012863580,
    11457135419348969979,
    9763749252098620046,
    16960553411078512574,
    15563877356819111679,
    14994736884583272463,
    9441297368950544394,
    14537646123432199168,
    9888547162215157388,
    18140215579194907366,
    18374682062228545019,
];

pub const ROOK_SHIFTS: [u64; 64] = [
    52, 52, 52, 52, 52, 52, 52, 52, 53, 53, 53, 54, 53, 53, 54, 53, 53, 54, 54, 54, 53, 53, 54, 53,
    53, 54, 53, 53, 54, 54, 54, 53, 52, 54, 53, 53, 53, 53, 54, 53, 52, 53, 54, 54, 53, 53, 54, 53,
    53, 54, 54, 54, 53, 53, 54, 53, 52, 53, 53, 53, 53, 53, 53, 52,
];
pub const BISHOP_SHIFTS: [u64; 64] = [
    58, 60, 59, 59, 59, 59, 60, 58, 60, 59, 59, 59, 59, 59, 59, 60, 59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 57, 57, 57, 59, 59,
    60, 60, 59, 59, 59, 59, 60, 60, 58, 60, 59, 59, 59, 59, 59, 58,
];

const fn rook_attacks_for_blockers(sq: usize, blockers: u64) -> u64 {
    let mut attacks = 0;
    let rank = sq / 8;
    let file = sq % 8;
    // North
    let mut r = rank + 1;
    while r < 8 {
        let target = r * 8 + file;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        r += 1;
    }
    // South
    let mut r = rank as i32 - 1;
    while r >= 0 {
        let target = (r * 8 + file as i32) as usize;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        r -= 1;
    }
    // East
    let mut f = file + 1;
    while f < 8 {
        let target = rank * 8 + f;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        f += 1;
    }
    // West
    let mut f = file as i32 - 1;
    while f >= 0 {
        let target = rank * 8 + f as usize;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        f -= 1;
    }
    attacks
}

const fn bishop_attacks_for_blockers(sq: usize, blockers: u64) -> u64 {
    let mut attacks = 0;
    let rank = sq / 8;
    let file = sq % 8;
    // North-East
    let mut r = rank + 1;
    let mut f = file + 1;
    while r < 8 && f < 8 {
        let target = r * 8 + f;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        r += 1;
        f += 1;
    }
    // North-West
    let mut r = rank + 1;
    let mut f = file as i32 - 1;
    while r < 8 && f >= 0 {
        let target = r * 8 + f as usize;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        r += 1;
        f -= 1;
    }
    // South-East
    let mut r = rank as i32 - 1;
    let mut f = file + 1;
    while r >= 0 && f < 8 {
        let target = (r * 8 + f as i32) as usize;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        r -= 1;
        f += 1;
    }
    // South-West
    let mut r = rank as i32 - 1;
    let mut f = file as i32 - 1;
    while r >= 0 && f >= 0 {
        let target = (r * 8 + f) as usize;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }
    attacks
}

const fn gen_rook_attacks() -> [[u64; 4096]; 64] {
    let mut table = [[0; 4096]; 64];
    let mut sq = 0;
    while sq < 64 {
        let mask = ROOK_MASKS[sq];
        let magic = ROOK_MAGIC_NUMBERS[sq];
        let shift = ROOK_SHIFTS[sq];
        let mut blockers = 0;
        loop {
            let attacks = rook_attacks_for_blockers(sq, blockers);
            let index = ((blockers.wrapping_mul(magic)) >> shift) as usize;
            table[sq][index] = attacks;
            blockers = (blockers.wrapping_sub(mask)) & mask;
            if blockers == 0 {
                break;
            }
        }
        sq += 1;
    }
    table
}

const fn gen_bishop_attacks() -> [[u64; 512]; 64] {
    let mut table = [[0; 512]; 64];
    let mut sq = 0;
    while sq < 64 {
        let mask = BISHOP_MASKS[sq];
        let magic = BISHOP_MAGIC_NUMBERS[sq];
        let shift = BISHOP_SHIFTS[sq];
        let mut blockers = 0;
        loop {
            let attacks = bishop_attacks_for_blockers(sq, blockers);
            let index = ((blockers.wrapping_mul(magic)) >> shift) as usize;
            table[sq][index] = attacks;
            blockers = (blockers.wrapping_sub(mask)) & mask;
            if blockers == 0 {
                break;
            }
        }
        sq += 1;
    }
    table
}

pub const ROOK_ATTACKS: [[u64; 4096]; 64] = gen_rook_attacks();

pub const BISHOP_ATTACKS: [[u64; 512]; 64] = gen_bishop_attacks();

const fn gen_king_attacks() -> [u64; 64] {
    let mut table = [0u64; 64];

    let mut sq = 0;
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;

        let mut attacks = 0u64;

        let mut dr = -1;
        while dr <= 1 {
            let mut df = -1;
            while df <= 1 {
                if dr == 0 && df == 0 {
                    df += 1;
                    continue;
                }

                let r = rank as i16 + dr;
                let f = file as i16 + df;

                if r >= 0 && r < 8 && f >= 0 && f < 8 {
                    let target = (r * 8 + f) as u64;
                    attacks |= 1u64 << target;
                }

                df += 1;
            }
            dr += 1;
        }
        table[sq] = attacks;
        sq += 1;
    }
    table
}

pub const KING_ATTACKS: [u64; 64] = gen_king_attacks();

const fn direction(from_i: i32, to_i: i32) -> i32 {
    let (from_rank, from_file) = (from_i / 8, from_i % 8);
    let (to_rank, to_file) = (to_i / 8, to_i % 8);

    let (dr, df) = (to_rank - from_rank, to_file - from_file);

    let (step_r, step_f) = (
        if dr == 0 {
            0
        } else if dr > 0 {
            1
        } else {
            -1
        },
        if df == 0 {
            0
        } else if df > 0 {
            1
        } else {
            -1
        },
    );

    if dr == 0 || df == 0 || dr.abs() == df.abs() {
        step_r * 8 + step_f
    } else {
        0
    }
}

const fn between_mask(from: usize, to: usize) -> u64 {
    let from_i = from as i32;
    let to_i = to as i32;

    let step = direction(from_i, to_i);

    if step == 0 {
        return 0;
    }

    let mut sq = from_i + step;
    let mut mask = 0u64;

    while sq != to_i {
        mask |= 1u64 << sq;
        sq += step;
    }

    mask
}

const fn gen_between() -> [[u64; 64]; 64] {
    let mut table = [[0u64; 64]; 64];

    let mut from = 0;
    while from < 64 {
        let mut to = 0;
        while to < 64 {
            table[from][to] = between_mask(from, to);
            to += 1;
        }
        from += 1;
    }

    table
}

pub const BETWEEN: [[u64; 64]; 64] = gen_between();
