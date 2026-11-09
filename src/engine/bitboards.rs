use crate::engine::types::Bitboard;

pub const ROOK_BLOCKER_POSSIBILITIES: usize = 4096;
pub const BISHOP_BLOCKER_POSSIBILITIES: usize = 512;

pub const FILE_A: Bitboard = Bitboard(0x01_01_01_01_01_01_01_01);
pub const FILE_B: Bitboard = Bitboard(0x02_02_02_02_02_02_02_02);
pub const FILE_G: Bitboard = Bitboard(0x40_40_40_40_40_40_40_40);
pub const FILE_H: Bitboard = Bitboard(0x80_80_80_80_80_80_80_80);
pub const RANK_2: Bitboard = Bitboard(0x00_00_00_00_00_00_FF_00);
pub const RANK_7: Bitboard = Bitboard(0x00_FF_00_00_00_00_00_00);

const fn gen_white_pawn_attacks() -> [Bitboard; 64] {
    let mut table = [Bitboard(0); 64];
    let mut sq = 0;
    while sq < 64 {
        let sq_mask = 1u64 << sq;
        let mut mask = 0u64;
        if sq < 56 {
            mask |= (sq_mask << 7) & !FILE_H.0;
            mask |= (sq_mask << 9) & !FILE_A.0;
        }
        table[sq] = Bitboard(mask);
        sq += 1;
    }
    table
}

pub const WHITE_PAWN_ATTACKS: [Bitboard; 64] = gen_white_pawn_attacks();

const fn gen_black_pawn_attacks() -> [Bitboard; 64] {
    let mut table = [Bitboard(0); 64];
    let mut sq = 0;
    while sq < 64 {
        let sq_mask = 1u64 << sq;
        let mut mask = 0u64;
        if sq > 7 {
            mask |= (sq_mask >> 7) & !FILE_A.0;
            mask |= (sq_mask >> 9) & !FILE_H.0;
        }
        table[sq] = Bitboard(mask);
        sq += 1;
    }
    table
}

pub const BLACK_PAWN_ATTACKS: [Bitboard; 64] = gen_black_pawn_attacks();

const fn gen_knight_attacks() -> [Bitboard; 64] {
    let mut table = [Bitboard(0); 64];
    let mut sq = 0_usize;

    while sq < 64 {
        let mask = 1u64 << sq;

        let attacks = ((mask << 17) & !FILE_A.0)
            | ((mask << 15) & !FILE_H.0)
            | ((mask << 10) & !FILE_A.0 & !FILE_B.0)
            | ((mask << 6) & !FILE_H.0 & !FILE_G.0)
            | ((mask >> 17) & !FILE_H.0)
            | ((mask >> 15) & !FILE_A.0)
            | ((mask >> 10) & !FILE_H.0 & !FILE_G.0)
            | ((mask >> 6) & !FILE_A.0 & !FILE_B.0);

        table[sq] = Bitboard(attacks);
        sq += 1;
    }

    table
}

pub const KNIGHT_ATTACKS: [Bitboard; 64] = gen_knight_attacks();

const fn gen_rook_masks() -> [Bitboard; 64] {
    let mut masks = [Bitboard(0); 64];
    let mut sq = 0;
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
        let mut r = rank;
        while r > 1 {
            r -= 1;
            mask |= 1u64 << (r * 8 + file);
        }

        // East
        let mut f = file + 1;
        while f < 7 {
            mask |= 1u64 << (rank * 8 + f);
            f += 1;
        }

        // West
        let mut f = file;
        while f > 1 {
            f -= 1;
            mask |= 1u64 << (rank * 8 + f);
        }

        masks[sq] = Bitboard(mask);
        sq += 1;
    }
    masks
}

const fn gen_bishop_masks() -> [Bitboard; 64] {
    let mut masks = [Bitboard(0); 64];
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
        let mut f = file;
        while r < 7 && f > 1 {
            f -= 1;
            mask |= 1u64 << (r * 8 + f);
            r += 1;
        }

        // South-East
        let mut r = rank;
        let mut f = file + 1;
        while r > 1 && f < 7 {
            r -= 1;
            mask |= 1u64 << (r * 8 + f);
            f += 1;
        }

        // South-West
        let mut r = rank;
        let mut f = file;
        while r > 1 && f > 1 {
            r -= 1;
            f -= 1;
            mask |= 1u64 << (r * 8 + f);
        }

        masks[sq] = Bitboard(mask);
        sq += 1;
    }
    masks
}

const ROOK_MASKS: [Bitboard; 64] = gen_rook_masks();

const BISHOP_MASKS: [Bitboard; 64] = gen_bishop_masks();

// Magics and shift from https://github.com/SebLague/Chess-Coding-Adventure/
const ROOK_MAGIC_NUMBERS: [u64; 64] = [
    468_374_916_371_625_120,
    18_428_729_537_625_841_661,
    2_531_023_729_696_186_408,
    6_093_370_314_119_450_896,
    13_830_552_789_156_493_815,
    16_134_110_446_239_088_507,
    12_677_615_322_350_354_425,
    5_404_321_144_167_858_432,
    2_111_097_758_984_580,
    18_428_720_740_584_907_710,
    17_293_734_603_602_787_839,
    4_938_760_079_889_530_922,
    7_699_325_603_589_095_390,
    9_078_693_890_218_258_431,
    578_149_610_753_690_728,
    9_496_543_503_900_033_792,
    1_155_209_038_552_629_657,
    9_224_076_274_589_515_780,
    1_835_781_998_207_181_184,
    509_120_063_316_431_138,
    16_634_043_024_132_535_807,
    18_446_673_631_917_146_111,
    9_623_686_630_121_410_312,
    4_648_737_361_302_392_899,
    738_591_182_849_868_645,
    1_732_936_432_546_219_272,
    2_400_543_327_507_449_856,
    5_188_164_365_601_475_096,
    10_414_575_345_181_196_316,
    1_162_492_212_166_789_136,
    9_396_848_738_060_210_946,
    622_413_200_109_881_612,
    7_998_357_718_131_801_918,
    7_719_627_227_008_073_923,
    16_181_433_497_662_382_080,
    18_441_958_655_457_754_079,
    1_267_153_596_645_440,
    18_446_726_464_209_379_263,
    1_214_021_438_038_606_600,
    4_650_128_814_733_526_084,
    9_656_144_899_867_951_104,
    18_444_421_868_610_287_615,
    3_695_311_799_139_303_489,
    10_597_006_226_145_476_632,
    18_436_046_904_206_950_398,
    18_446_726_472_933_277_663,
    3_458_977_943_764_860_944,
    39_125_045_590_687_766,
    9_227_453_435_446_560_384,
    6_476_955_465_732_358_656,
    1_270_314_852_531_077_632,
    2_882_448_553_461_416_064,
    11_547_238_928_203_796_481,
    1_856_618_300_822_323_264,
    2_573_991_788_166_144,
    4_936_544_992_551_831_040,
    13_690_941_749_405_253_631,
    15_852_669_863_439_351_807,
    18_302_628_748_190_527_413,
    12_682_135_449_552_027_479,
    13_830_554_446_930_287_982,
    18_302_628_782_487_371_519,
    7_924_083_509_981_736_956,
    4_734_295_326_018_586_370,
];
const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
    16_509_839_532_542_417_919,
    14_391_803_910_955_204_223,
    1_848_771_770_702_627_364,
    347_925_068_195_328_958,
    5_189_277_761_285_652_493,
    3_750_937_732_777_063_343,
    18_429_848_470_517_967_340,
    17_870_072_066_711_748_607,
    16_715_520_087_474_960_373,
    2_459_353_627_279_607_168,
    7_061_705_824_611_107_232,
    8_089_129_053_103_260_512,
    7_414_579_821_471_224_013,
    9_520_647_030_890_121_554,
    17_142_940_634_164_625_405,
    9_187_037_984_654_475_102,
    4_933_695_867_036_173_873,
    3_035_992_416_931_960_321,
    15_052_160_563_071_165_696,
    5_876_081_268_917_084_809,
    1_153_484_746_652_717_320,
    6_365_855_841_584_713_735,
    2_463_646_859_659_644_933,
    1_453_259_901_463_176_960,
    9_808_859_429_721_908_488,
    2_829_141_021_535_244_552,
    576_619_101_540_319_252,
    5_804_014_844_877_275_314,
    4_774_660_099_383_771_136,
    328_785_038_479_458_864,
    2_360_590_652_863_023_124,
    569_550_314_443_282,
    17_563_974_527_758_635_567,
    11_698_101_887_533_589_556,
    5_764_964_460_729_992_192,
    6_953_579_832_080_335_136,
    1_318_441_160_687_747_328,
    8_090_717_009_753_444_376,
    16_751_172_641_200_572_929,
    5_558_033_503_209_157_252,
    17_100_156_536_247_493_656,
    7_899_286_223_048_400_564,
    4_845_135_427_956_654_145,
    2_368_485_888_099_072,
    2_399_033_289_953_272_320,
    6_976_678_428_284_034_058,
    3_134_241_565_013_966_284,
    8_661_609_558_376_259_840,
    17_275_805_361_393_991_679,
    15_391_050_065_516_657_151,
    11_529_206_229_534_274_423,
    9_876_416_274_250_600_448,
    16_432_792_402_597_134_585,
    11_975_705_497_012_863_580,
    11_457_135_419_348_969_979,
    9_763_749_252_098_620_046,
    16_960_553_411_078_512_574,
    15_563_877_356_819_111_679,
    14_994_736_884_583_272_463,
    9_441_297_368_950_544_394,
    14_537_646_123_432_199_168,
    9_888_547_162_215_157_388,
    18_140_215_579_194_907_366,
    18_374_682_062_228_545_019,
];

const ROOK_SHIFTS: [u64; 64] = [
    52, 52, 52, 52, 52, 52, 52, 52, 53, 53, 53, 54, 53, 53, 54, 53, 53, 54, 54, 54, 53, 53, 54, 53,
    53, 54, 53, 53, 54, 54, 54, 53, 52, 54, 53, 53, 53, 53, 54, 53, 52, 53, 54, 54, 53, 53, 54, 53,
    53, 54, 54, 54, 53, 53, 54, 53, 52, 53, 53, 53, 53, 53, 53, 52,
];
const BISHOP_SHIFTS: [u64; 64] = [
    58, 60, 59, 59, 59, 59, 60, 58, 60, 59, 59, 59, 59, 59, 59, 60, 59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 57, 57, 57, 59, 59,
    60, 60, 59, 59, 59, 59, 60, 60, 58, 60, 59, 59, 59, 59, 59, 58,
];

const fn gen_rook_attacks_for_blockers(sq: usize, blockers: u64) -> u64 {
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
    let mut r = rank;
    while r > 0 {
        r -= 1;
        let target = r * 8 + file;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
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
    let mut f = file;
    while f > 0 {
        f -= 1;
        let target = rank * 8 + f;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
    }
    attacks
}

const fn gen_bishop_attacks_for_blockers(sq: usize, blockers: u64) -> u64 {
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
    let mut f = file;
    while r < 8 && f > 0 {
        f -= 1;
        let target = r * 8 + f;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        r += 1;
    }
    // South-East
    let mut r = rank;
    let mut f = file + 1;
    while r > 0 && f < 8 {
        r -= 1;
        let target = r * 8 + f;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
        f += 1;
    }
    // South-West
    let mut r = rank;
    let mut f = file;
    while r > 0 && f > 0 {
        r -= 1;
        f -= 1;
        let target = r * 8 + f;
        attacks |= 1 << target;
        if blockers & (1 << target) != 0 {
            break;
        }
    }
    attacks
}

#[allow(clippy::large_stack_arrays)]
#[allow(clippy::large_stack_frames)]
const fn gen_rook_attacks() -> [Bitboard; ROOK_BLOCKER_POSSIBILITIES * 64] {
    let mut table = [Bitboard(0); ROOK_BLOCKER_POSSIBILITIES * 64];
    let mut sq = 0;
    while sq < 64 {
        let base = sq * ROOK_BLOCKER_POSSIBILITIES;
        let mask = ROOK_MASKS[sq];
        let magic = ROOK_MAGIC_NUMBERS[sq];
        let shift = ROOK_SHIFTS[sq];
        let mut blockers = 0;
        loop {
            let attacks = gen_rook_attacks_for_blockers(sq, blockers);
            #[allow(clippy::cast_possible_truncation)]
            let index = ((blockers.wrapping_mul(magic)) >> shift) as usize;
            table[base + index] = Bitboard(attacks);
            blockers = (blockers.wrapping_sub(mask.0)) & mask.0;
            if blockers == 0 {
                break;
            }
        }
        sq += 1;
    }
    table
}

#[allow(clippy::large_stack_arrays)]
#[allow(clippy::large_stack_frames)]
const fn gen_bishop_attacks() -> [Bitboard; BISHOP_BLOCKER_POSSIBILITIES * 64] {
    let mut table = [Bitboard(0); BISHOP_BLOCKER_POSSIBILITIES * 64];
    let mut sq = 0;
    while sq < 64 {
        let base = sq * BISHOP_BLOCKER_POSSIBILITIES;
        let mask = BISHOP_MASKS[sq];
        let magic = BISHOP_MAGIC_NUMBERS[sq];
        let shift = BISHOP_SHIFTS[sq];
        let mut blockers = 0;
        loop {
            let attacks = gen_bishop_attacks_for_blockers(sq, blockers);
            #[allow(clippy::cast_possible_truncation)]
            let index = ((blockers.wrapping_mul(magic)) >> shift) as usize;
            table[base + index] = Bitboard(attacks);
            blockers = (blockers.wrapping_sub(mask.0)) & mask.0;
            if blockers == 0 {
                break;
            }
        }
        sq += 1;
    }
    table
}

pub static ROOK_ATTACKS: [Bitboard; ROOK_BLOCKER_POSSIBILITIES * 64] = gen_rook_attacks();

pub static BISHOP_ATTACKS: [Bitboard; BISHOP_BLOCKER_POSSIBILITIES * 64] = gen_bishop_attacks();

#[inline(always)]
pub fn rook_attacks(sq_usize: usize, occ: Bitboard) -> Bitboard {
    let mask = ROOK_MASKS[sq_usize];
    let blockers = occ & mask;
    let index =
        (blockers.wrapping_mul(ROOK_MAGIC_NUMBERS[sq_usize]) >> ROOK_SHIFTS[sq_usize]) as usize;
    ROOK_ATTACKS[sq_usize * ROOK_BLOCKER_POSSIBILITIES + index]
}

#[inline(always)]
pub fn bishop_attacks(sq_usize: usize, occ: Bitboard) -> Bitboard {
    let mask = BISHOP_MASKS[sq_usize];
    let blockers = occ & mask;
    let index =
        (blockers.wrapping_mul(BISHOP_MAGIC_NUMBERS[sq_usize]) >> BISHOP_SHIFTS[sq_usize]) as usize;
    BISHOP_ATTACKS[sq_usize * BISHOP_BLOCKER_POSSIBILITIES + index]
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
const fn gen_king_attacks() -> [Bitboard; 64] {
    let mut table = [Bitboard(0); 64];

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
                    let target = (r * 8 + f) as usize;
                    attacks |= 1u64 << target;
                }

                df += 1;
            }
            dr += 1;
        }
        table[sq] = Bitboard(attacks);
        sq += 1;
    }
    table
}

pub const KING_ATTACKS: [Bitboard; 64] = gen_king_attacks();

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

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
const fn between_mask(from: usize, to: usize) -> Bitboard {
    #[allow(clippy::cast_possible_wrap)]
    let from_i = from as i32;
    let to_i = to as i32;

    let step = direction(from_i, to_i);

    if step == 0 {
        return Bitboard(0);
    }

    let mut sq = from_i + step;
    let mut mask = 0u64;

    while sq != to_i {
        mask |= 1u64 << sq;
        sq += step;
    }

    Bitboard(mask)
}

#[allow(clippy::large_stack_arrays)]
const fn gen_between() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard(0); 64]; 64];

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

pub static BETWEEN: [[Bitboard; 64]; 64] = gen_between();
