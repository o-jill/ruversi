use super::*;
use rand::Rng;
use std::{fs, io::{BufReader, BufRead}};
use aligned_vec::AVec;

#[cfg(target_arch="x86_64")]
use std::arch::x86_64;

#[cfg(target_arch="aarch64")]
use std::arch::aarch64::*;


/*
 * input: NUMCELL * NUMCELL + 1(teban) + 2(fixedstones) + 1
 * hidden: 8 + 1
 * output: 1
 */
const N_INPUT : usize = bitboard::CELL_2D + 1 + 2;
const N_HIDDEN : usize = 128;
pub const N_HIDDEN2 : usize = 16;
const N_OUTPUT : usize = 1;
const N_WEIGHT_TEBAN : usize =  bitboard::CELL_2D * N_HIDDEN;
const N_WEIGHT_FIXST_B : usize = N_WEIGHT_TEBAN + N_HIDDEN;
const N_WEIGHT_FIXST_W : usize = N_WEIGHT_FIXST_B + N_HIDDEN;
const N_WEIGHT_INPUTBIAS : usize = N_WEIGHT_FIXST_W + N_HIDDEN;
const N_WEIGHT_LAYER1 : usize = N_WEIGHT_INPUTBIAS + N_HIDDEN;
const N_WEIGHT_LAYER1BIAS : usize = N_WEIGHT_LAYER1 + N_HIDDEN * N_HIDDEN2;
const N_WEIGHT_LAYER2 : usize = N_WEIGHT_LAYER1BIAS + N_HIDDEN2;
const N_WEIGHT_LAYER2BIAS : usize = N_WEIGHT_LAYER2 + N_HIDDEN2;
const N_WEIGHT : usize =
  (N_INPUT + 1) * N_HIDDEN + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;

const N_WEIGHT_PAD :usize = N_WEIGHT.div_ceil(8) * 8;
pub const N_PROGRESS_DIV : usize = 3;  // 序盤中盤終盤

#[allow(dead_code)]
const WSZV1 : usize = (bitboard::CELL_2D + 1 + 1) * 4 + 4 + 1;
#[allow(dead_code)]
const WSZV2 : usize = WSZV1;
#[allow(dead_code)]
const WSZV3 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * 4 + 4 + 1;
#[allow(dead_code)]
const WSZV4 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * 8 + 8 + 1;
#[allow(dead_code)]
const WSZV5 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * 16 + 16 + 1;
#[allow(dead_code)]
const WSZV6 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * N_HIDDEN + N_HIDDEN + 1;
#[allow(dead_code)]
const WSZV7 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * 32
        + (32 + 1) * 16 + 16 + 1;
const WSZV8 : usize = (bitboard::CELL_2D + 1 + 2 + 1) * N_HIDDEN
        + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;
const WSZV9 : usize = WSZV8;

// v2
// 8/8/1A6/2Ab3/2C3/8/8/8 w
// val:-273.121 val:Some(-273.1215), 268965 nodes. []b6@@b5[]c6@@a7[]a5@@a6[]a8 60msec
// 8/8/1A6/2Ab3/2aB3/1a6/8/8 b
// val:-3.506 val:Some(-3.5055861), 334278 nodes. @@c3[]c2@@d1[]c1@@b1[]a4@@a2 80msec

#[derive(PartialEq)]
enum EvalFile{
    Unknown,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
}

impl EvalFile {
    #[allow(dead_code)]
    pub fn to_str(&self) -> &str {
        match self {
            EvalFile::Unknown => {"unknown eval file format."},
            EvalFile::V1 => {"# 65-4-1"},
            EvalFile::V2 => {"# 64+1-4-1"},
            EvalFile::V3 => {"# 64+1+2-4-1"},
            EvalFile::V4 => {"# 64+1+2-8-1"},
            EvalFile::V5 => {"# 64+1+2-16-1"},
            EvalFile::V6 => {"# 64+1+2-32-1"},
            EvalFile::V7 => {"# 64+1+2-32-16-1"},
            EvalFile::V8 => {"# 64+1+2-128-16-1"},
            EvalFile::V9 => {"# 3x 64+1+2-128-16-1"},
        }
    }

    pub fn from(txt : &str) -> Option<EvalFile> {
        match txt {
            "# 65-4-1" => Some(EvalFile::V1),
            "# 64+1-4-1" => Some(EvalFile::V2),
            "# 64+1+2-4-1" => Some(EvalFile::V3),
            "# 64+1+2-8-1" => Some(EvalFile::V4),
            "# 64+1+2-16-1" => Some(EvalFile::V5),
            "# 64+1+2-32-1" => Some(EvalFile::V6),
            "# 64+1+2-32-16-1" => Some(EvalFile::V7),
            "# 64+1+2-128-16-1" => Some(EvalFile::V8),
            "# 3x 64+1+2-128-16-1" => Some(EvalFile::V9),
            _ => None
        }
    }
}

/**
 * conversion table from bits to f32.
 */
#[repr(align(32))]
pub struct Bit2F32 {
    table : [f32 ; 256 * 8]
}

impl Bit2F32 {
    /// # Safety
    ///
    /// # Arguments
    /// - `idx` - index of the table.
    pub unsafe fn addr(&self, idx : usize) -> *const f32 {
        self.table.as_ptr().add(idx * 8)
    }
}

const TBL8_BIT2F32 : Bit2F32 = Bit2F32 {
    table : [
        0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32,
        1f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32,
        1f32, 1f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 0f32,
        1f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 1f32, 0f32, 0f32, 0f32, 0f32, 0f32,
        1f32, 1f32, 1f32, 0f32, 0f32, 0f32, 0f32, 0f32,
        0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32,
        1f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32,
        1f32, 1f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 1f32, 0f32, 0f32, 0f32, 0f32,
        1f32, 0f32, 1f32, 1f32, 0f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32, 0f32,
        1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32, 0f32,
        0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32,
        1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32,
        1f32, 1f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 0f32, 1f32, 0f32, 0f32, 0f32,
        1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32, 0f32,
        1f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32, 0f32,
        0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 0f32, 0f32,
        1f32, 0f32, 0f32, 1f32, 1f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 0f32, 0f32,
        1f32, 1f32, 0f32, 1f32, 1f32, 0f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32,
        1f32, 0f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32,
        0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32,
        1f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32,
        0f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32,
        1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32,
        1f32, 1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 0f32, 0f32, 1f32, 0f32, 0f32,
        1f32, 0f32, 1f32, 0f32, 0f32, 1f32, 0f32, 0f32,
        0f32, 1f32, 1f32, 0f32, 0f32, 1f32, 0f32, 0f32,
        1f32, 1f32, 1f32, 0f32, 0f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 0f32, 1f32, 0f32, 1f32, 0f32, 0f32,
        1f32, 0f32, 0f32, 1f32, 0f32, 1f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 0f32,
        1f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32,
        1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32,
        0f32, 1f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32,
        1f32, 1f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 0f32,
        1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 0f32, 1f32, 1f32, 0f32, 0f32,
        1f32, 1f32, 0f32, 0f32, 1f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 0f32,
        1f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 0f32,
        0f32, 1f32, 1f32, 0f32, 1f32, 1f32, 0f32, 0f32,
        1f32, 1f32, 1f32, 0f32, 1f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 0f32, 1f32, 1f32, 1f32, 0f32, 0f32,
        1f32, 0f32, 0f32, 1f32, 1f32, 1f32, 0f32, 0f32,
        0f32, 1f32, 0f32, 1f32, 1f32, 1f32, 0f32, 0f32,
        1f32, 1f32, 0f32, 1f32, 1f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32,
        1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32,
        0f32, 1f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32,
        1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32,
        0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
        1f32, 0f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
        0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
        1f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32, 0f32,
        1f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32, 0f32,
        0f32, 1f32, 1f32, 0f32, 0f32, 0f32, 1f32, 0f32,
        1f32, 1f32, 1f32, 0f32, 0f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 1f32, 0f32,
        1f32, 0f32, 0f32, 1f32, 0f32, 0f32, 1f32, 0f32,
        0f32, 1f32, 0f32, 1f32, 0f32, 0f32, 1f32, 0f32,
        1f32, 1f32, 0f32, 1f32, 0f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 1f32, 1f32, 0f32, 0f32, 1f32, 0f32,
        1f32, 0f32, 1f32, 1f32, 0f32, 0f32, 1f32, 0f32,
        0f32, 1f32, 1f32, 1f32, 0f32, 0f32, 1f32, 0f32,
        1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 1f32, 0f32,
        1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 1f32, 0f32,
        0f32, 1f32, 0f32, 0f32, 1f32, 0f32, 1f32, 0f32,
        1f32, 1f32, 0f32, 0f32, 1f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32,
        1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32,
        0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32,
        1f32, 1f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32,
        1f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32,
        0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32,
        1f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 1f32, 1f32, 1f32, 0f32, 1f32, 0f32,
        1f32, 0f32, 1f32, 1f32, 1f32, 0f32, 1f32, 0f32,
        0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 1f32, 0f32,
        1f32, 1f32, 1f32, 1f32, 1f32, 0f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32,
        1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32,
        0f32, 1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32,
        1f32, 1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32,
        0f32, 0f32, 1f32, 0f32, 0f32, 1f32, 1f32, 0f32,
        1f32, 0f32, 1f32, 0f32, 0f32, 1f32, 1f32, 0f32,
        0f32, 1f32, 1f32, 0f32, 0f32, 1f32, 1f32, 0f32,
        1f32, 1f32, 1f32, 0f32, 0f32, 1f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32,
        1f32, 0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32,
        0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32,
        1f32, 1f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32,
        0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32, 0f32,
        1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32, 0f32,
        0f32, 1f32, 1f32, 1f32, 0f32, 1f32, 1f32, 0f32,
        1f32, 1f32, 1f32, 1f32, 0f32, 1f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 0f32, 1f32, 1f32, 1f32, 0f32,
        1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 1f32, 0f32,
        0f32, 1f32, 0f32, 0f32, 1f32, 1f32, 1f32, 0f32,
        1f32, 1f32, 0f32, 0f32, 1f32, 1f32, 1f32, 0f32,
        0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 1f32, 0f32,
        1f32, 0f32, 1f32, 0f32, 1f32, 1f32, 1f32, 0f32,
        0f32, 1f32, 1f32, 0f32, 1f32, 1f32, 1f32, 0f32,
        1f32, 1f32, 1f32, 0f32, 1f32, 1f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 1f32, 1f32, 1f32, 1f32, 0f32,
        1f32, 0f32, 0f32, 1f32, 1f32, 1f32, 1f32, 0f32,
        0f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 0f32,
        1f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 0f32,
        0f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32, 0f32,
        1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32, 0f32,
        0f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 0f32,
        1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 0f32,
        0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 1f32,
        1f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 1f32,
        0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 0f32, 1f32,
        1f32, 1f32, 0f32, 0f32, 0f32, 0f32, 0f32, 1f32,
        0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
        1f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
        0f32, 1f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
        1f32, 1f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32,
        0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        1f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        0f32, 1f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        1f32, 1f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        0f32, 0f32, 1f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        1f32, 0f32, 1f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        0f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 0f32, 1f32,
        0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 1f32,
        1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 1f32,
        0f32, 1f32, 0f32, 0f32, 1f32, 0f32, 0f32, 1f32,
        1f32, 1f32, 0f32, 0f32, 1f32, 0f32, 0f32, 1f32,
        0f32, 0f32, 1f32, 0f32, 1f32, 0f32, 0f32, 1f32,
        1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 0f32, 1f32,
        0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32, 1f32,
        1f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32, 1f32,
        0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 0f32, 1f32,
        1f32, 0f32, 0f32, 1f32, 1f32, 0f32, 0f32, 1f32,
        0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 0f32, 1f32,
        1f32, 1f32, 0f32, 1f32, 1f32, 0f32, 0f32, 1f32,
        0f32, 0f32, 1f32, 1f32, 1f32, 0f32, 0f32, 1f32,
        1f32, 0f32, 1f32, 1f32, 1f32, 0f32, 0f32, 1f32,
        0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 1f32,
        1f32, 1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 1f32,
        0f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 1f32,
        1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 1f32,
        0f32, 1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 1f32,
        1f32, 1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 1f32,
        0f32, 0f32, 1f32, 0f32, 0f32, 1f32, 0f32, 1f32,
        1f32, 0f32, 1f32, 0f32, 0f32, 1f32, 0f32, 1f32,
        0f32, 1f32, 1f32, 0f32, 0f32, 1f32, 0f32, 1f32,
        1f32, 1f32, 1f32, 0f32, 0f32, 1f32, 0f32, 1f32,
        0f32, 0f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32,
        1f32, 0f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32,
        0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32,
        1f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32,
        0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 1f32,
        1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 1f32,
        0f32, 1f32, 1f32, 1f32, 0f32, 1f32, 0f32, 1f32,
        1f32, 1f32, 1f32, 1f32, 0f32, 1f32, 0f32, 1f32,
        0f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32,
        1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32,
        0f32, 1f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32,
        1f32, 1f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32,
        0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32,
        1f32, 0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32,
        0f32, 1f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32,
        1f32, 1f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32,
        0f32, 0f32, 0f32, 1f32, 1f32, 1f32, 0f32, 1f32,
        1f32, 0f32, 0f32, 1f32, 1f32, 1f32, 0f32, 1f32,
        0f32, 1f32, 0f32, 1f32, 1f32, 1f32, 0f32, 1f32,
        1f32, 1f32, 0f32, 1f32, 1f32, 1f32, 0f32, 1f32,
        0f32, 0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 1f32,
        1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 1f32,
        0f32, 1f32, 1f32, 1f32, 1f32, 1f32, 0f32, 1f32,
        1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 0f32, 1f32,
        0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 1f32, 1f32,
        1f32, 0f32, 0f32, 0f32, 0f32, 0f32, 1f32, 1f32,
        0f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 1f32,
        1f32, 1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 1f32,
        0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32, 1f32,
        1f32, 0f32, 1f32, 0f32, 0f32, 0f32, 1f32, 1f32,
        0f32, 1f32, 1f32, 0f32, 0f32, 0f32, 1f32, 1f32,
        1f32, 1f32, 1f32, 0f32, 0f32, 0f32, 1f32, 1f32,
        0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 1f32, 1f32,
        1f32, 0f32, 0f32, 1f32, 0f32, 0f32, 1f32, 1f32,
        0f32, 1f32, 0f32, 1f32, 0f32, 0f32, 1f32, 1f32,
        1f32, 1f32, 0f32, 1f32, 0f32, 0f32, 1f32, 1f32,
        0f32, 0f32, 1f32, 1f32, 0f32, 0f32, 1f32, 1f32,
        1f32, 0f32, 1f32, 1f32, 0f32, 0f32, 1f32, 1f32,
        0f32, 1f32, 1f32, 1f32, 0f32, 0f32, 1f32, 1f32,
        1f32, 1f32, 1f32, 1f32, 0f32, 0f32, 1f32, 1f32,
        0f32, 0f32, 0f32, 0f32, 1f32, 0f32, 1f32, 1f32,
        1f32, 0f32, 0f32, 0f32, 1f32, 0f32, 1f32, 1f32,
        0f32, 1f32, 0f32, 0f32, 1f32, 0f32, 1f32, 1f32,
        1f32, 1f32, 0f32, 0f32, 1f32, 0f32, 1f32, 1f32,
        0f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 1f32,
        1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 1f32,
        0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 1f32, 1f32,
        1f32, 1f32, 1f32, 0f32, 1f32, 0f32, 1f32, 1f32,
        0f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32,
        1f32, 0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32,
        0f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32,
        1f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32,
        0f32, 0f32, 1f32, 1f32, 1f32, 0f32, 1f32, 1f32,
        1f32, 0f32, 1f32, 1f32, 1f32, 0f32, 1f32, 1f32,
        0f32, 1f32, 1f32, 1f32, 1f32, 0f32, 1f32, 1f32,
        1f32, 1f32, 1f32, 1f32, 1f32, 0f32, 1f32, 1f32,
        0f32, 0f32, 0f32, 0f32, 0f32, 1f32, 1f32, 1f32,
        1f32, 0f32, 0f32, 0f32, 0f32, 1f32, 1f32, 1f32,
        0f32, 1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 1f32,
        1f32, 1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 1f32,
        0f32, 0f32, 1f32, 0f32, 0f32, 1f32, 1f32, 1f32,
        1f32, 0f32, 1f32, 0f32, 0f32, 1f32, 1f32, 1f32,
        0f32, 1f32, 1f32, 0f32, 0f32, 1f32, 1f32, 1f32,
        1f32, 1f32, 1f32, 0f32, 0f32, 1f32, 1f32, 1f32,
        0f32, 0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 1f32,
        1f32, 0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 1f32,
        0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 1f32, 1f32,
        1f32, 1f32, 0f32, 1f32, 0f32, 1f32, 1f32, 1f32,
        0f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32, 1f32,
        1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 1f32, 1f32,
        0f32, 1f32, 1f32, 1f32, 0f32, 1f32, 1f32, 1f32,
        1f32, 1f32, 1f32, 1f32, 0f32, 1f32, 1f32, 1f32,
        0f32, 0f32, 0f32, 0f32, 1f32, 1f32, 1f32, 1f32,
        1f32, 0f32, 0f32, 0f32, 1f32, 1f32, 1f32, 1f32,
        0f32, 1f32, 0f32, 0f32, 1f32, 1f32, 1f32, 1f32,
        1f32, 1f32, 0f32, 0f32, 1f32, 1f32, 1f32, 1f32,
        0f32, 0f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32,
        1f32, 0f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32,
        0f32, 1f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32,
        1f32, 1f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32,
        0f32, 0f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32,
        1f32, 0f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32,
        0f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32,
        1f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32,
        0f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32,
        1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32,
        0f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32,
        1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32, 1f32,
    ],
};

const MEM_ALIGN : usize = 64;

#[repr(align(32))]
pub struct Weight {
    // 64xH1 + H1 + H1x2 + H1 + H1 x (H2+1) + H2 + 1
    pub weight : AVec<f32>,
    // H1x64 + H1 + H1x2 + H1 + H1 x (H2+1) + H2 + 1
    vweight : AVec<f32>
}

impl Default for Weight {
    fn default() -> Self {
        let mut w = Self::new();
        w.exchange();
        w
    }
}

impl Weight {
    pub fn new() -> Weight {
        Weight {
            weight: {
                let mut w =
                    AVec::with_capacity(
                        MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV);
                unsafe {w.set_len(w.capacity());}
                w
            },
            vweight: AVec::with_capacity(
                MEM_ALIGN, N_WEIGHT_PAD * N_PROGRESS_DIV)
        }
    }

    fn exchange(&mut self) {
        unsafe {self.vweight.set_len(self.weight.len());}
        for p in 0..N_PROGRESS_DIV {
            let offset = p * N_WEIGHT_PAD;
            let wei = &mut self.weight[offset..offset + N_WEIGHT_PAD];
            let vwei = &mut self.vweight[offset..offset + N_WEIGHT_PAD];
            for (i, &w) in wei.iter().enumerate().take(bitboard::CELL_2D * N_HIDDEN) {
                let group = i % bitboard::CELL_2D;
                let x = i / bitboard::CELL_2D;
                let idx = x + group * N_HIDDEN;
                vwei[idx] = w;
            }
        }
    }

    pub fn init(&mut self) {
        let mut rng = rand::thread_rng();
        let range =
            f64::sqrt(6.0) /
                f64::sqrt((N_INPUT + N_HIDDEN + N_HIDDEN2 + N_OUTPUT) as f64);

        for a in self.weight.iter_mut() {
            *a = (rng.gen::<f64>() * 2.0 * range - range) as f32;
        }
        self.exchange();
    }

    /// fill zero.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.weight.iter_mut().for_each(|m| *m = 0.0);
    }

    pub fn wban(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset..]
    }

    pub fn wbanv(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.vweight[offset..]
    }

    pub fn wteban(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_TEBAN..offset + N_WEIGHT_FIXST_W]
    }

    pub fn wfixedstones(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_FIXST_B..offset + N_WEIGHT_INPUTBIAS]
    }

    #[allow(dead_code)]
    pub fn wfixedstone_b(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_FIXST_B..offset + N_WEIGHT_FIXST_W]
    }

    #[allow(dead_code)]
    pub fn wfixedstone_w(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_FIXST_W..offset + N_WEIGHT_INPUTBIAS]
    }

    pub fn wibias(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_INPUTBIAS..offset + N_WEIGHT_LAYER1]
    }

    pub fn wlayer1(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER1..offset + N_WEIGHT_LAYER1BIAS]
    }

    pub fn wl1bias(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER1BIAS..offset + N_WEIGHT_LAYER2]
    }

    pub fn wlayer2(&self, progress : usize) -> &[f32] {
        let offset = progress * N_WEIGHT_PAD;
        &self.weight[offset + N_WEIGHT_LAYER2..offset + N_WEIGHT_LAYER2BIAS]
    }

    pub fn wl2bias(&self, progress : usize) -> f32 {
        let offset = progress * N_WEIGHT_PAD;
        self.weight[offset + N_WEIGHT - 1]
    }

    /// read eval table from a file.
    /// 
    /// # Arguments
    /// - `path` file path to a eval table.  
    ///   "RANDOM" is a special text to fill every paramerter w/ random numbers.
    pub fn read(&mut self, path : &str) -> Result<(), String> {
        if path == "RANDOM" {
            self.init();
            return Ok(());
        }
        let mut format = EvalFile::Unknown;
        let file = File::open(path);
        if let Err(e) = file {
            eprintln!("path: {path}");
            return Err(e.to_string());
        }

        let mut idx = 0;
        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) => {
                    if l.starts_with("#") {
                        if format != EvalFile::Unknown {continue;}

                        if let Some(fmt) = EvalFile::from(&l) {
                            format = fmt
                        }
                        continue;
                    }

                    match format {
                        EvalFile::V1 => {return self.readv1(&l)},
                        EvalFile::V2 => {return self.readv2(&l)},
                        EvalFile::V3 => {return self.readv3(&l)},
                        EvalFile::V4 => {return self.readv4(&l)},
                        EvalFile::V5 => {return self.readv5(&l)},
                        EvalFile::V6 => {return self.readv6(&l)},
                        EvalFile::V7 => {return self.readv7(&l)},
                        EvalFile::V8 => {return self.readv8(&l)},
                        EvalFile::V9 => {
                            self.readv9(&l, idx)?;
                            idx += 1;
                            if idx >= N_PROGRESS_DIV {return Ok(());}
                        },
                        _ => {}
                    }
                },
                Err(err) => {return Err(err.to_string())}
            }
        }

        Err("no weight".to_string())
    }

    fn readv1(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v1 format is not supported any more."))
    }

    fn readv2(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v2 format is not supported any more."))
    }

    fn readv3(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v3 format is not supported any more."))
    }

    fn readv4(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v4 format is not supported any more."))
    }

    fn readv5(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v5 format is not supported any more."))
    }

    fn readv6(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v6 format is not supported any more."))
    }

    fn readv7(&mut self, _line : &str) -> Result<(), String> {
        Err(String::from("v7 format is not supported any more."))
    }

    fn readv8(&mut self, line : &str) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> = csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV8 != nsz {
            return Err(format!("size mismatch v8:{WSZV8} != {nsz}"));
        }
        for prgs in 0..N_PROGRESS_DIV {
            let offset = prgs * N_WEIGHT_PAD;
            self.weight[offset..offset + N_WEIGHT].copy_from_slice(&newtable);
        }
        self.exchange();
        // println!("v8:{:?}", self.weight);
        Ok(())
    }

    fn readv9(&mut self, line : &str, progress : usize) -> Result<(), String> {
        let csv = line.split(",").collect::<Vec<_>>();
        let newtable : Vec<f32> =
                csv.iter().map(|&a| a.parse::<f32>().unwrap()).collect();
        let nsz = newtable.len();
        if WSZV9 != nsz {
            return Err(format!("size mismatch v9:{WSZV9} != {nsz}"));
        }

        let offset = progress * N_WEIGHT_PAD;
        self.weight[offset..offset + N_WEIGHT].copy_from_slice(&newtable);
        self.exchange();
        // println!("v9:{:?}", self.weight);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn writev9(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(
            format!("{}\n", EvalFile::V9.to_str()).as_bytes()).unwrap();
        for prgs in 0..N_PROGRESS_DIV {
            let offset = prgs * N_WEIGHT_PAD;
            let w = &self.weight[offset..offset + N_WEIGHT];
            let sv = w.iter().map(|a| a.to_string()).collect::<Vec<String>>();
            f.write_all((sv.join(",") + "¥n").as_bytes()).unwrap();
        }
    }

    pub fn copy(&mut self, src : &Weight) {
        self.weight.copy_from_slice(&src.weight);
        self.exchange();
    }

    #[allow(dead_code)]
    pub fn evaluatev9bb_old(&self, ban : &bitboard::BitBoard) -> f32 {
        if ban.is_full() || ban.is_passpass() {
            return ban.countf32();
        }

        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wban(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);
        let mut hid = [0f32 ; N_HIDDEN];
        for (i, h) in hid.iter_mut().enumerate() {
            let w1 = &ow[i * bitboard::CELL_2D .. (i + 1) * bitboard::CELL_2D];
            let mut hidsum : f32 = 0.0;  // wdc[i];
            let mut bit = bitboard::LSB_CELL;
            for y in 0..bitboard::NUMCELL {
                for x in 0..bitboard::NUMCELL {
                    let w = w1[x + y * bitboard::NUMCELL];
                    hidsum +=
                    if (bit & black) != 0 {w}
                    else if (bit & white) != 0 {-w}
                    else {0.0};
                    bit_right!(bit);
                }
            }
            hidsum = teban.mul_add(wtbn[i], hidsum);
            hidsum = wfs[i].mul_add(fs.0 as f32, hidsum);
            hidsum = wfs[i + N_HIDDEN].mul_add(fs.1 as f32, hidsum + wdc[i]);
            //relu
            *h = if hidsum > 0f32 {hidsum} else {0f32};
        }

        let mut sum = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let whdc = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = whdc[i];
            for (j, h1) in hid.iter().enumerate() {
                hidsum2 = h1.mul_add(wh[j + i * N_HIDDEN], hidsum2);
                // hidsum2 += h1 * wh[j + i * N_HIDDEN];
            }
            // relu
            sum += if hidsum2 > 0.0 {wh2[i] * hidsum2} else {0.0};
        }
        sum
    }

    pub fn evaluatev9bb(&self, ban : &bitboard::BitBoard) -> f32 {
        if ban.is_full() || ban.is_passpass() {
            return ban.countf32();
        }

        let prgs = ban.progress();
        let mut black = ban.black;
        let mut white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wbanv(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);
        let mut hid = [0f32 ; N_HIDDEN];
        for y in 0..bitboard::NUMCELL {
            for x in 0..bitboard::NUMCELL {
                let bit = bitboard::LSB_CELL;
                let c = (black & bit) as i32 - (white & bit) as i32;
                let c = c as f32;
                black >>= 1;
                white >>= 1;
                for (h, w) in hid.iter_mut().zip(
                    ow.iter().skip((x + y * bitboard::NUMCELL) * N_HIDDEN)) {
                    *h += c * w;
                }
            }
        }
        for (i, h) in hid.iter_mut().enumerate() {
            let mut hidsum = teban.mul_add(wtbn[i], *h);
            hidsum = wfs[i].mul_add(fs.0 as f32, hidsum);
            hidsum = wfs[i + N_HIDDEN].mul_add(fs.1 as f32, hidsum + wdc[i]);
            // relu
            *h = hidsum.max(0f32);
        }

        let mut sum = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let whdc = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = whdc[i];
            for (j, h1) in hid.iter().enumerate() {
                hidsum2 = h1.mul_add(wh[j + i * N_HIDDEN], hidsum2);
                // hidsum2 += h1 * wh[j + i * N_HIDDEN];
            }
            // relu
            sum += if hidsum2 > 0.0 {wh2[i] * hidsum2} else {0.0};
        }
        sum
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev9bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        if ban.is_full() || ban.is_passpass() {
            return ban.countf32();
        }

        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wban(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);

        let mut cells : Vec<f32> = Vec::with_capacity(bitboard::CELL_2D);
        unsafe {
            let c_ptr  = cells.spare_capacity_mut().as_mut_ptr() as *mut f32;
            let bit4 = 0xf;
            for idx in (0..bitboard::CELL_2D).step_by(16) {
                let bi1 = bit4 & (black >> idx) as usize;
                let wi1 = bit4 & (white >> idx) as usize;
                let bi2 = bit4 & (black >> (idx + 4)) as usize;
                let wi2 = bit4 & (white >> (idx + 4)) as usize;
                let bi3 = bit4 & (black >> (idx + 8)) as usize;
                let wi3 = bit4 & (white >> (idx + 8)) as usize;
                let bi4 = bit4 & (black >> (idx + 12)) as usize;
                let wi4 = bit4 & (white >> (idx + 12)) as usize;
                let b41 = x86_64::_mm_load_ps(TBL8_BIT2F32.addr(bi1));
                let b42 = x86_64::_mm_load_ps(TBL8_BIT2F32.addr(bi2));
                let b43 = x86_64::_mm_load_ps(TBL8_BIT2F32.addr(bi3));
                let b44 = x86_64::_mm_load_ps(TBL8_BIT2F32.addr(bi4));
                let w41 = x86_64::_mm_load_ps(TBL8_BIT2F32.addr(wi1));
                let w42 = x86_64::_mm_load_ps(TBL8_BIT2F32.addr(wi2));
                let w43 = x86_64::_mm_load_ps(TBL8_BIT2F32.addr(wi3));
                let w44 = x86_64::_mm_load_ps(TBL8_BIT2F32.addr(wi4));
                let c1 = x86_64::_mm_sub_ps(b41, w41);
                let c2 = x86_64::_mm_sub_ps(b42, w42);
                let c3 = x86_64::_mm_sub_ps(b43, w43);
                let c4 = x86_64::_mm_sub_ps(b44, w44);
                x86_64::_mm_storeu_ps(c_ptr.add(idx), c1);
                x86_64::_mm_storeu_ps(c_ptr.add(idx + 4), c2);
                x86_64::_mm_storeu_ps(c_ptr.add(idx + 8), c3);
                x86_64::_mm_storeu_ps(c_ptr.add(idx + 12), c4);
            }
            cells.set_len(bitboard::CELL_2D);
        }
        let mut hid = [0f32 ; N_HIDDEN];
        const N : usize = 8;

        for i in (0..N_HIDDEN).step_by(N) {
            let mut sum44 : [f32 ; N * 4] = [0f32 ; N * 4];

            const M : usize = 16;
            for idx in (0..bitboard::CELL_2D).step_by(M) {
                unsafe {
                    let c1 = x86_64::_mm_loadu_ps(cells.as_ptr().add(idx));
                    let c2 = x86_64::_mm_loadu_ps(cells.as_ptr().add(idx + 4));
                    let c3 = x86_64::_mm_loadu_ps(cells.as_ptr().add(idx + 8));
                    let c4 = x86_64::_mm_loadu_ps(cells.as_ptr().add(idx + 12));

                    for n in 0..N {
                        let w1 = &ow[(i + n) * bitboard::CELL_2D .. (i + n + 1) * bitboard::CELL_2D];

                        let x41 = x86_64::_mm_load_ps(w1.as_ptr().add(idx));
                        let x42 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 4));
                        let x43 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 8));
                        let x44 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 12));

                        let m1 = x86_64::_mm_mul_ps(c1, x41);
                        let m2 = x86_64::_mm_mul_ps(c2, x42);
                        // let m3 = x86_64::_mm_mul_ps(c3, x43);
                        // let m4 = x86_64::_mm_mul_ps(c4, x44);
                        // let sum12 = x86_64::_mm_add_ps(m1, m2);
                        // let sum34 = x86_64::_mm_add_ps(m3, m4);
                        let sum12 = x86_64::_mm_fmadd_ps(c3, x43, m1);
                        let sum34 = x86_64::_mm_fmadd_ps(c4, x44, m2);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        let res4 = x86_64::_mm_loadu_ps(sum44.as_ptr().add(n * 4));
                        let sum4 = x86_64::_mm_add_ps(res4, sum1234);
                        x86_64::_mm_storeu_ps(sum44.as_mut_ptr().add(n * 4), sum4);
                    }
                }
            }

            unsafe {
                let mut x1 = x86_64::_mm_loadu_ps(sum44[0..].as_ptr());
                let mut x2 = x86_64::_mm_loadu_ps(sum44[4..].as_ptr());
                let mut x3 = x86_64::_mm_loadu_ps(sum44[8..].as_ptr());
                let mut x4 = x86_64::_mm_loadu_ps(sum44[12..].as_ptr());
                let mut x5 = x86_64::_mm_loadu_ps(sum44[16..].as_ptr());
                let mut x6 = x86_64::_mm_loadu_ps(sum44[20..].as_ptr());
                let mut x7 = x86_64::_mm_loadu_ps(sum44[24..].as_ptr());
                let mut x8 = x86_64::_mm_loadu_ps(sum44[28..].as_ptr());

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);
                x86_64::_MM_TRANSPOSE4_PS(&mut x5, &mut x6, &mut x7, &mut x8);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);
                let h12 = x86_64::_mm_add_ps(x5, x6);
                let h34 = x86_64::_mm_add_ps(x7, x8);
                let h5678 = x86_64::_mm_add_ps(h12, h34);
                // teban
                let wtbn1 = x86_64::_mm_load_ps(wtbn[i..].as_ptr());
                let wtbn2 = x86_64::_mm_load_ps(wtbn[i + 4..].as_ptr());
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn1, tbn);
                let tbn42 = x86_64::_mm_mul_ps(wtbn2, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                let h5678 = x86_64::_mm_add_ps(h5678, tbn42);
                // fixed stones
                let wfsb4 = x86_64::_mm_load_ps(wfs[i..].as_ptr());
                let wfsb42 = x86_64::_mm_load_ps(wfs[i + 4..].as_ptr());
                let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm_mul_ps(wfsb4, fsb);
                let fsb42 = x86_64::_mm_mul_ps(wfsb42, fsb);
                let wfsw4 = x86_64::_mm_load_ps(wfs[i + N_HIDDEN..].as_ptr());
                let wfsw42 = x86_64::_mm_load_ps(wfs[i + N_HIDDEN + 4..].as_ptr());
                let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm_mul_ps(wfsw4, fsw);
                let fsw42 = x86_64::_mm_mul_ps(wfsw42, fsw);
                let fsbw = x86_64::_mm_add_ps(fsb4, fsw4);
                let fsbw2 = x86_64::_mm_add_ps(fsb42, fsw42);
                let h1234 = x86_64::_mm_add_ps(h1234, fsbw);
                let h5678 = x86_64::_mm_add_ps(h5678, fsbw2);
                // dc
                let wdc4 = x86_64::_mm_load_ps(wdc[i..].as_ptr());
                let wdc42 = x86_64::_mm_load_ps(wdc[i + 4..].as_ptr());
                let h1234 = x86_64::_mm_add_ps(h1234, wdc4);
                let h5678 = x86_64::_mm_add_ps(h5678, wdc42);
                // relu
                let zero = x86_64::_mm_setzero_ps();
                let y4 = x86_64::_mm_max_ps(h1234, zero);
                let y42 = x86_64::_mm_max_ps(h5678, zero);

                x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i), y4);
                x86_64::_mm_storeu_ps(hid.as_mut_ptr().add(i + 4), y42);
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        let mut hid2 = [0f32 ; N_HIDDEN2];
        let mut sum4 = [0f32 ; 4 * N_HIDDEN2];
        for j in (0..N_HIDDEN).step_by(16) {
            unsafe {
                let x1 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j));
                let x2 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 4));
                let x3 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 8));
                let x4 = x86_64::_mm_loadu_ps(hid.as_ptr().add(j + 12));
                for i in 0..N_HIDDEN2 {
                    let idx = i * N_HIDDEN + j;
                    let w1 = x86_64::_mm_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 4));
                    let w3 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 8));
                    let w4 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 12));
                    let mul1 = x86_64::_mm_mul_ps(x1, w1);
                    let mul2 = x86_64::_mm_mul_ps(x2, w2);
                    // let mul3 = x86_64::_mm_mul_ps(x3, w3);
                    // let mul4 = x86_64::_mm_mul_ps(x4, w4);
                    // let s12 = x86_64::_mm_add_ps(mul1, mul2);
                    // let s34 = x86_64::_mm_add_ps(mul3, mul4);
                    let s12 = x86_64::_mm_fmadd_ps(x3, w3, mul1);
                    let s34 = x86_64::_mm_fmadd_ps(x4, w4, mul2);
                    let s1234 = x86_64::_mm_add_ps(s12, s34);
                    let s4 = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4));
                    let s4 = x86_64::_mm_add_ps(s1234, s4);
                    x86_64::_mm_storeu_ps(sum4.as_mut_ptr().add(i * 4), s4);
                }
            }
        }
        for i in (0..N_HIDDEN2).step_by(4) {
            unsafe {
                let a = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4));
                let b = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 4));
                let c = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 8));
                let d = x86_64::_mm_loadu_ps(sum4.as_ptr().add(i * 4 + 12));
                let a0c0a1c1 = x86_64::_mm_unpacklo_ps(a, c);
                let b0d0b1d1 = x86_64::_mm_unpacklo_ps(b, d);
                let a2c2a3c3 = x86_64::_mm_unpackhi_ps(a, c);
                let b2d2b3d3 = x86_64::_mm_unpackhi_ps(b, d);
                let a0 = x86_64::_mm_unpacklo_ps(a0c0a1c1, b0d0b1d1);
                let a1 = x86_64::_mm_unpackhi_ps(a0c0a1c1, b0d0b1d1);
                let a2 = x86_64::_mm_unpacklo_ps(a2c2a3c3, b2d2b3d3);
                let a3 = x86_64::_mm_unpackhi_ps(a2c2a3c3, b2d2b3d3);
                let s1 = x86_64::_mm_add_ps(a0, a1);
                let s2 = x86_64::_mm_add_ps(a2, a3);
                let s3 = x86_64::_mm_add_ps(s1, s2);

                let dc = x86_64::_mm_load_ps(wdc1.as_ptr().add(i));
                let s4 = x86_64::_mm_add_ps(s3, dc);
                x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(i), s4);
            }
        }
        unsafe {  // relu
            let h1 = x86_64::_mm_loadu_ps(hid2.as_ptr());
            let h2 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(4));
            let h3 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(8));
            let h4 = x86_64::_mm_loadu_ps(hid2.as_ptr().add(12));
            let zero = x86_64::_mm_setzero_ps();
            let h1 = x86_64::_mm_max_ps(h1, zero);
            let h2 = x86_64::_mm_max_ps(h2, zero);
            let h3 = x86_64::_mm_max_ps(h3, zero);
            let h4 = x86_64::_mm_max_ps(h4, zero);
            let wh21 = x86_64::_mm_load_ps(wh2.as_ptr());
            let wh22 = x86_64::_mm_load_ps(wh2.as_ptr().add(4));
            let wh23 = x86_64::_mm_load_ps(wh2.as_ptr().add(8));
            let wh24 = x86_64::_mm_load_ps(wh2.as_ptr().add(12));

            let y1 = x86_64::_mm_mul_ps(wh21, h1);
            let y2 = x86_64::_mm_mul_ps(wh22, h2);
            let y3 = x86_64::_mm_mul_ps(wh23, h3);
            let y4 = x86_64::_mm_mul_ps(wh24, h4);
            let y12 = x86_64::_mm_add_ps(y1, y2);
            let y34 = x86_64::_mm_add_ps(y3, y4);
            let y1234 = x86_64::_mm_add_ps(y12, y34);
            x86_64::_mm_storeu_ps(hid2.as_mut_ptr(), y1234);
        }
        for h in hid2.iter().take(4) {
            res += h;
        }
        res
    }

    #[cfg(target_arch="aarch64")]
    pub fn evaluatev9bb_simd_mul(&self, ban : &bitboard::BitBoard) -> f32 {
        if ban.is_full() || ban.is_passpass() {
            return ban.countf32();
        }

        let prgs = ban.progress();
        let mut black = ban.black;
        let mut white = ban.white;
        let teban = ban.teban as f32;

        let (fsb, fsw) = ban.fixedstones();

        let ow = self.wbanv(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);
        const N : usize = 16;
        let mut hid = [0f32 ; N_HIDDEN];
        // cells
        for idx in 0..bitboard::CELL_2D {
            let bit = bitboard::LSB_CELL;
            let c = (black & bit) as i32 - (white & bit) as i32;
            black >>= 1;
            white >>= 1;
            if c == 0 {continue;}

            let c4 = unsafe {vdupq_n_f32(c as f32)};
            let we1 = &ow[idx * N_HIDDEN .. ];
            for i in (0..N_HIDDEN).step_by(N) {
                unsafe {
                    let w = vld1q_f32_x4(we1.as_ptr().add(i));
                    let w1 = vmulq_f32(c4, w.0);
                    let w2 = vmulq_f32(c4, w.1);
                    let w3 = vmulq_f32(c4, w.2);
                    let w4 = vmulq_f32(c4, w.3);
                    let h = vld1q_f32_x4(hid.as_ptr().add(i));
                    let w1 = vaddq_f32(w1, h.0);
                    let w2 = vaddq_f32(w2, h.1);
                    let w3 = vaddq_f32(w3, h.2);
                    let w4 = vaddq_f32(w4, h.3);
                    vst1q_f32_x4(hid.as_mut_ptr().add(i),
                        float32x4x4_t(w1, w2, w3, w4));
                }
            }
        }
        for i in (0..N_HIDDEN).step_by(N) {
            unsafe {
                let sum4 = vld1q_f32_x4(hid.as_ptr().add(i));

                let tbn = vmovq_n_f32(teban);
                let wtb = vld1q_f32_x4(wtbn.as_ptr().add(i));
                let sum41 = vmlaq_f32(sum4.0, tbn, wtb.0);
                let sum42 = vmlaq_f32(sum4.1, tbn, wtb.1);
                let sum43 = vmlaq_f32(sum4.2, tbn, wtb.2);
                let sum44 = vmlaq_f32(sum4.3, tbn, wtb.3);

                let fsb4 = vmovq_n_f32(fsb as f32);
                let wfsb = vld1q_f32_x4(wfs.as_ptr().add(i));
                let sum41 = vmlaq_f32(sum41, fsb4, wfsb.0);
                let sum42 = vmlaq_f32(sum42, fsb4, wfsb.1);
                let sum43 = vmlaq_f32(sum43, fsb4, wfsb.2);
                let sum44 = vmlaq_f32(sum44, fsb4, wfsb.3);

                let fsw4 = vmovq_n_f32(fsw as f32);
                let wfsw = vld1q_f32_x4(wfs.as_ptr().add(i + N_HIDDEN));
                let sum41 = vmlaq_f32(sum41, fsw4, wfsw.0);
                let sum42 = vmlaq_f32(sum42, fsw4, wfsw.1);
                let sum43 = vmlaq_f32(sum43, fsw4, wfsw.2);
                let sum44 = vmlaq_f32(sum44, fsw4, wfsw.3);

                let wdc4 = vld1q_f32_x4(wdc.as_ptr().add(i));
                let sum41 = vaddq_f32(sum41, wdc4.0);
                let sum42 = vaddq_f32(sum42, wdc4.1);
                let sum43 = vaddq_f32(sum43, wdc4.2);
                let sum44 = vaddq_f32(sum44, wdc4.3);
                // relu
                let zero = vmovq_n_f32(0.0);
                let rl1 = vmaxq_f32(zero, sum41);
                let rl2 = vmaxq_f32(zero, sum42);
                let rl3 = vmaxq_f32(zero, sum43);
                let rl4 = vmaxq_f32(zero, sum44);
                vst1q_f32_x4(hid.as_mut_ptr().add(i), float32x4x4_t(rl1, rl2, rl3, rl4));
                // vst1q_f32(hid.as_mut_ptr().add(i), rl1);
                // vst1q_f32(hid.as_mut_ptr().add(i + 4), rl2);
                // vst1q_f32(hid.as_mut_ptr().add(i + 8), rl3);
                // vst1q_f32(hid.as_mut_ptr().add(i + 12), rl4);
            }
        }
        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);
        let mut hid2 = [0f32 ; N_HIDDEN2];
        hid2.copy_from_slice(wdc1);
        for j in (0..N_HIDDEN).step_by(32) {
            unsafe {
                let inp = vld1q_f32_x4(hid.as_ptr().add(j));
                for (i, h2) in hid2.iter_mut().enumerate() {
                    let wei = vld1q_f32_x4(wh.as_ptr().add(i * N_HIDDEN + j));
                    let mul0 = vmulq_f32(inp.0, wei.0);
                    let mul1 = vmulq_f32(inp.1, wei.1);
                    let mul2 = vmlaq_f32(mul0, inp.2, wei.2);
                    let mul3 = vmlaq_f32(mul1, inp.3, wei.3);
                    let add4 = vaddq_f32(mul2, mul3);

                    let inp = vld1q_f32_x4(hid.as_ptr().add(j + 16));
                    let wei = vld1q_f32_x4(wh.as_ptr().add(i * N_HIDDEN + j + 16));
                    let mul0 = vmulq_f32(inp.0, wei.0);
                    let mul1 = vmulq_f32(inp.1, wei.1);
                    let mul2 = vmlaq_f32(mul0, inp.2, wei.2);
                    let mul3 = vmlaq_f32(mul1, inp.3, wei.3);
                    let add42 = vaddq_f32(mul2, mul3);
                    let add4 = vaddq_f32(add4, add42);
                    *h2 += vaddvq_f32(add4);
                }
            }
        }
        for (i, _h) in hid2.iter().enumerate().step_by(16) {
            unsafe {
                let inp = vld1q_f32_x4(hid2.as_ptr().add(i));
                // relu
                let zero = vmovq_n_f32(0.0);
                let inp0 = vmaxq_f32(zero, inp.0);
                let inp1 = vmaxq_f32(zero, inp.1);
                let inp2 = vmaxq_f32(zero, inp.2);
                let inp3 = vmaxq_f32(zero, inp.3);

                let wei = vld1q_f32_x4(wh2.as_ptr().add(i));
                let mul0 = vmulq_f32(wei.0, inp0);
                let mul1 = vmulq_f32(wei.1, inp1);
                let mul2 = vmulq_f32(wei.2, inp2);
                let mul3 = vmulq_f32(wei.3, inp3);
                let mul12 = vaddq_f32(mul0, mul1);
                let mul34 = vaddq_f32(mul2, mul3);
                let add4 = vaddq_f32(mul12, mul34);
                res += vaddvq_f32(add4);
            }
        }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev9bb_simdavx(&self, ban : &bitboard::BitBoard) -> f32 {
        if ban.is_full() || ban.is_passpass() {
            return ban.countf32();
        }

        let prgs = ban.progress();
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wban(prgs);
        let wtbn = self.wteban(prgs);
        let wfs = self.wfixedstones(prgs);
        let wdc = self.wibias(prgs);
        const N : usize = 16;
        let mut hid = [0f32 ; N_HIDDEN];
        let mut sumn = [0f32 ; N];
        let mut cells : Vec<f32> = Vec::with_capacity(bitboard::CELL_2D);
        unsafe {
            let c_ptr = cells.spare_capacity_mut().as_mut_ptr() as *mut f32;
            let bit8 = 0xff;
            for idx in (0..bitboard::CELL_2D).step_by(32) {
                let bi1 = bit8 & (black >> idx) as usize;
                let wi1 = bit8 & (white >> idx) as usize;
                let bi2 = bit8 & (black >> (idx + 8)) as usize;
                let wi2 = bit8 & (white >> (idx + 8)) as usize;
                let bi3 = bit8 & (black >> (idx + 16)) as usize;
                let wi3 = bit8 & (white >> (idx + 16)) as usize;
                let bi4 = bit8 & (black >> (idx + 24)) as usize;
                let wi4 = bit8 & (white >> (idx + 24)) as usize;
                let b81 = x86_64::_mm256_load_ps(TBL8_BIT2F32.addr(bi1));
                let b82 = x86_64::_mm256_load_ps(TBL8_BIT2F32.addr(bi2));
                let b83 = x86_64::_mm256_load_ps(TBL8_BIT2F32.addr(bi3));
                let b84 = x86_64::_mm256_load_ps(TBL8_BIT2F32.addr(bi4));
                let w81 = x86_64::_mm256_load_ps(TBL8_BIT2F32.addr(wi1));
                let w82 = x86_64::_mm256_load_ps(TBL8_BIT2F32.addr(wi2));
                let w83 = x86_64::_mm256_load_ps(TBL8_BIT2F32.addr(wi3));
                let w84 = x86_64::_mm256_load_ps(TBL8_BIT2F32.addr(wi4));
                let f81 = x86_64::_mm256_sub_ps(b81, w81);
                let f82 = x86_64::_mm256_sub_ps(b82, w82);
                let f83 = x86_64::_mm256_sub_ps(b83, w83);
                let f84 = x86_64::_mm256_sub_ps(b84, w84);
                x86_64::_mm256_storeu_ps(c_ptr.add(idx), f81);
                x86_64::_mm256_storeu_ps(c_ptr.add(idx + 8), f82);
                x86_64::_mm256_storeu_ps(c_ptr.add(idx + 16), f83);
                x86_64::_mm256_storeu_ps(c_ptr.add(idx + 24), f84);
            }
            cells.set_len(bitboard::CELL_2D);
        }
        for hidx in (0..N_HIDDEN).step_by(N) {
            for m in (0..N).step_by(8) {
                let mut sum88 = [0f32 ; N * 8 / 2];
                const M : usize = 32;
                for idx in (0..bitboard::CELL_2D).step_by(M) {
                    unsafe {
                        let f81 = x86_64::_mm256_loadu_ps(
                                cells.as_ptr().add(idx));
                        let f82 = x86_64::_mm256_loadu_ps(
                                cells.as_ptr().add(idx + 8));
                        let f83 = x86_64::_mm256_loadu_ps(
                                cells.as_ptr().add(idx + 16));
                        let f84 = x86_64::_mm256_loadu_ps(
                                cells.as_ptr().add(idx + 24));

                        for n in 0..N / 2 {
                            let index = hidx + m + n;
                            let w1 = &ow[index * bitboard::CELL_2D .. (index + 1) * bitboard::CELL_2D];
                            let mut sum8 = x86_64::_mm256_loadu_ps(
                                    sum88[n * 8..].as_ptr());

                            let x81 = x86_64::_mm256_load_ps(
                                w1.as_ptr().add(idx));
                            let x82 = x86_64::_mm256_load_ps(
                                w1.as_ptr().add(idx + 8));
                            let x83 = x86_64::_mm256_load_ps(
                                w1.as_ptr().add(idx + 16));
                            let x84 = x86_64::_mm256_load_ps(
                                w1.as_ptr().add(idx + 24));

                            if true {  // fma
                                sum8 = x86_64::_mm256_fmadd_ps(x81, f81, sum8);
                                sum8 = x86_64::_mm256_fmadd_ps(x82, f82, sum8);
                                sum8 = x86_64::_mm256_fmadd_ps(x83, f83, sum8);
                                sum8 = x86_64::_mm256_fmadd_ps(x84, f84, sum8);
                            } else {
                                let mul1 = x86_64::_mm256_mul_ps(x81, f81);
                                let mul2 = x86_64::_mm256_mul_ps(x82, f82);
                                let mul3 = x86_64::_mm256_mul_ps(x83, f83);
                                let mul4 = x86_64::_mm256_mul_ps(x84, f84);

                                let sum12 = x86_64::_mm256_add_ps(mul1, mul2);
                                let sum34 = x86_64::_mm256_add_ps(mul3, mul4);
                                let sum1234 = x86_64::_mm256_add_ps(sum12, sum34);
                                sum8 = x86_64::_mm256_add_ps(sum8, sum1234);
                            }
                            x86_64::_mm256_storeu_ps(
                                sum88[n * 8..].as_mut_ptr(), sum8);
                        }
                    }
                }
                // sum88->sumn
                // transpose
                unsafe {
                    let x1 = x86_64::_mm256_load_ps(sum88.as_ptr());
                    let x2 = x86_64::_mm256_load_ps(sum88.as_ptr().add(8));
                    let x3 = x86_64::_mm256_load_ps(sum88.as_ptr().add(16));
                    let x4 = x86_64::_mm256_load_ps(sum88.as_ptr().add(24));
                    let x5 = x86_64::_mm256_load_ps(sum88.as_ptr().add(32));
                    let x6 = x86_64::_mm256_load_ps(sum88.as_ptr().add(40));
                    let x7 = x86_64::_mm256_load_ps(sum88.as_ptr().add(48));
                    let x8 = x86_64::_mm256_load_ps(sum88.as_ptr().add(56));

                    let xl12 = x86_64::_mm256_unpacklo_ps(x1, x2);
                    let xh12 = x86_64::_mm256_unpackhi_ps(x1, x2);
                    let xl34 = x86_64::_mm256_unpacklo_ps(x3, x4);
                    let xh34 = x86_64::_mm256_unpackhi_ps(x3, x4);
                    let xl56 = x86_64::_mm256_unpacklo_ps(x5, x6);
                    let xh56 = x86_64::_mm256_unpackhi_ps(x5, x6);
                    let xl78 = x86_64::_mm256_unpacklo_ps(x7, x8);
                    let xh78 = x86_64::_mm256_unpackhi_ps(x7, x8);

                    let x12 = x86_64::_mm256_add_ps(xl12, xh12);
                    let x34 = x86_64::_mm256_add_ps(xl34, xh34);
                    let x56 = x86_64::_mm256_add_ps(xl56, xh56);
                    let x78 = x86_64::_mm256_add_ps(xl78, xh78);

                    let x1234 = x86_64::_mm256_shuffle_ps(x12, x34, 0x44);
                    let xabcd = x86_64::_mm256_shuffle_ps(x12, x34, 0xee);
                    let x5678 = x86_64::_mm256_shuffle_ps(x56, x78, 0x44);
                    let xefgh = x86_64::_mm256_shuffle_ps(x56, x78, 0xee);

                    let xabcd = x86_64::_mm256_add_ps(x1234, xabcd);
                    let xefgh = x86_64::_mm256_add_ps(x5678, xefgh);

                    let x1234 = x86_64::_mm256_permute2f128_ps(xabcd, xefgh, 0x20);
                    let x5678 = x86_64::_mm256_permute2f128_ps(xabcd, xefgh, 0x31);

                    let h18 = x86_64::_mm256_add_ps(x1234, x5678);
                    // sum
                    x86_64::_mm256_storeu_ps(sumn.as_mut_ptr().add(m), h18);
                }
            }

            unsafe {
                let x1 = x86_64::_mm256_load_ps(sumn.as_ptr());
                let x2 = x86_64::_mm256_load_ps(sumn.as_ptr().add(8));
                // teban
                let wtbn1 = x86_64::_mm256_load_ps(wtbn.as_ptr().add(hidx));
                let wtbn2 = x86_64::_mm256_load_ps(wtbn.as_ptr().add(hidx + 8));
                let tbn = x86_64::_mm256_set1_ps(teban);
                let tbn1 = x86_64::_mm256_mul_ps(wtbn1, tbn);
                let tbn2 = x86_64::_mm256_mul_ps(wtbn2, tbn);
                let h1 = x86_64::_mm256_add_ps(x1, tbn1);
                let h2 = x86_64::_mm256_add_ps(x2, tbn2);
                // fixed stones
                let wfsb1 = x86_64::_mm256_load_ps(wfs.as_ptr().add(hidx));
                let wfsb2 = x86_64::_mm256_load_ps(wfs.as_ptr().add(hidx + 8));
                let fsb = x86_64::_mm256_set1_ps(fs.0 as f32);
                let fsb1 = x86_64::_mm256_mul_ps(wfsb1, fsb);
                let fsb2 = x86_64::_mm256_mul_ps(wfsb2, fsb);
                let wfsw1 = x86_64::_mm256_load_ps(
                    wfs.as_ptr().add(hidx + N_HIDDEN));
                let wfsw2 = x86_64::_mm256_load_ps(
                    wfs.as_ptr().add(hidx + N_HIDDEN + 8));
                let fsw = x86_64::_mm256_set1_ps(fs.1 as f32);
                let fsw1 = x86_64::_mm256_mul_ps(wfsw1, fsw);
                let fsw2 = x86_64::_mm256_mul_ps(wfsw2, fsw);
                let fsbw1 = x86_64::_mm256_add_ps(fsb1, fsw1);
                let fsbw2 = x86_64::_mm256_add_ps(fsb2, fsw2);
                let h1 = x86_64::_mm256_add_ps(h1, fsbw1);
                let h2 = x86_64::_mm256_add_ps(h2, fsbw2);
                // dc
                let wdc1 = x86_64::_mm256_load_ps(wdc.as_ptr().add(hidx));
                let wdc2 = x86_64::_mm256_load_ps(wdc.as_ptr().add(hidx + 8));
                let h1234 = x86_64::_mm256_add_ps(h1, wdc1);
                let h5678 = x86_64::_mm256_add_ps(h2, wdc2);
                // relu
                let zero = x86_64::_mm256_setzero_ps();
                let y41 = x86_64::_mm256_max_ps(zero, h1234);
                let y42 = x86_64::_mm256_max_ps(zero, h5678);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx), y41);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(hidx + 8), y42);
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias(prgs);
        let wh = self.wlayer1(prgs);
        let wdc1 = self.wl1bias(prgs);
        let wh2 = self.wlayer2(prgs);

        let mut hid2 = [0f32 ; N_HIDDEN2];
        hid2.copy_from_slice(wdc1);
        let mut sumhn = [0f32 ; N_HIDDEN2 * 4 * 2];
        for j in (0..N_HIDDEN).step_by(32) {
            unsafe {
                let x1 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j));
                let x2 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 8));
                let x3 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 16));
                let x4 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 24));
                for i in 0..N_HIDDEN2 {
                    let idx = i * N_HIDDEN + j;
                    let w1 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 8));
                    let w3 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 16));
                    let w4 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 24));
                    let mul1 = x86_64::_mm256_mul_ps(x1, w1);
                    let mul2 = x86_64::_mm256_mul_ps(x2, w2);
                    // let mul3 = x86_64::_mm256_mul_ps(x3, w3);
                    // let mul4 = x86_64::_mm256_mul_ps(x4, w4);
                    // let s12 = x86_64::_mm256_add_ps(mul1, mul2);
                    // let s34 = x86_64::_mm256_add_ps(mul3, mul4);
                    let s12 = x86_64::_mm256_fmadd_ps(x3, w3, mul1);
                    let s34 = x86_64::_mm256_fmadd_ps(x4, w4, mul2);
                    let s1234 = x86_64::_mm256_add_ps(s12, s34);
                    x86_64::_mm256_storeu_ps(
                            sumhn.as_mut_ptr().add(i * 8), s1234);
                }
                for (k, _hn) in sumhn.iter().enumerate().step_by(32) {
                    use std::arch::x86_64::_mm256_extractf128_ps;

                    let a = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k));  // a0~a7
                    let b = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 8));  // a8~a15
                    let c = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 16));  // b0~b7
                    let d = x86_64::_mm256_loadu_ps(
                            sumhn.as_ptr().add(k + 24));  // b8~b15
                    let a0c0 = x86_64::_mm256_unpacklo_ps(a, b);
                    let b0d0 = x86_64::_mm256_unpacklo_ps(c, d);
                    let a2c2 = x86_64::_mm256_unpackhi_ps(a, b);
                    let b2d2 = x86_64::_mm256_unpackhi_ps(c, d);
                    let s1 = x86_64::_mm256_add_ps(a0c0, a2c2);
                    let s2 = x86_64::_mm256_add_ps(b0d0, b2d2);
                    let t1 = x86_64::_mm256_shuffle_ps(s1, s2,
                            0b01000100/*(1 << 6) | (0 << 4) | (1 << 2) | 0*/);
                    let t2 = x86_64::_mm256_shuffle_ps(s1, s2,
                            0b11101110/*(3 << 6) | (2 << 4) | (3 << 2) | 2*/);
                    let s3 = x86_64::_mm256_add_ps(t1, t2);
                    let s4 = _mm256_extractf128_ps(s3, 1);
                    let s5 = x86_64::_mm_add_ps(
                            s4, x86_64::_mm256_castps256_ps128(s3));
                    let hn2 = x86_64::_mm_loadu_ps(
                            hid2.as_mut_ptr().add(k / 8));
                    let s6 = x86_64::_mm_add_ps(s5, hn2);
                    x86_64::_mm_storeu_ps(hid2.as_mut_ptr().add(k / 8), s6);
                }
            }
        }
        unsafe {  // relu
            let x1 = x86_64::_mm256_loadu_ps(hid2.as_ptr());
            let x2 = x86_64::_mm256_loadu_ps(hid2.as_ptr().add(8));
            let zero = x86_64::_mm256_setzero_ps();
            let h1 = x86_64::_mm256_max_ps(zero, x1);
            let h2 = x86_64::_mm256_max_ps(zero, x2);
            let w1 = x86_64::_mm256_load_ps(wh2.as_ptr());
            let w2 = x86_64::_mm256_load_ps(wh2.as_ptr().add(8));
            let y1 = x86_64::_mm256_mul_ps(h1, w1);
            let y2 = x86_64::_mm256_mul_ps(h2, w2);
            let y3 = x86_64::_mm256_add_ps(y1, y2);
            let s1 = x86_64::_mm256_castps256_ps128(y3);
            let s2 = x86_64::_mm256_extractf128_ps(y3, 1);
            let s4 = x86_64::_mm_add_ps(s1, s2);
            x86_64::_mm_store_ps(hid2.as_mut_ptr(), s4);
        }
        for h in hid2.iter().take(4) {
            res += h;
        }
        res
    }
}

#[allow(dead_code)]
fn dbg_assert_eq_vec(va : &[f32], vb : &[f32]) -> bool {
    for (a, b) in va.iter().zip(vb.iter()) {
        if (a - b).abs() >= 2e-6 {
            println!("| {a} - {b} | >= 2e-6...");
            return false;
        }
    }
    true
}

#[allow(dead_code)]
fn dbg_assert_eq(a : &f32, b : &f32) -> bool {
    if (a - b).abs() >= 2e-6 {
        println!("| {a} - {b} | >= 2e-6...");
        return false;
    }
    true
}

#[cfg(target_arch="x86_64")]
#[test]
fn testweight() {
    let rfens = [
        "h/H/h/H/h/H/h/H b",
        "h/H/h/H/h/H/h/H w",
        "H/h/H/h/H/h/H/h b",
        "H/h/H/h/H/h/H/h w",
        "h/H/8/H/h/H/h/H b",
        "h/H/h/8/h/H/h/H w",
        "H/h/H/h/8/h/H/h b",
        "H/h/H/h/H/8/H/h w",
        "aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa b",
        "aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA w",
        "8/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa b",
        "aAaAaAaA/AaAaAaAa/8/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/8/aAaAaAaA/AaAaAaAa/aAaAaAaA w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/8/aAaAaAaA b",
        "aAaAaAaA/8/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/8/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA w",
        "aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa b",
        "1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa w",
        "AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1 w",
        "A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA b",
        "aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A b",
        "aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA w",
    ];
    for rfen in rfens.iter() {
        let bban = bitboard::BitBoard::from(rfen).unwrap();
        let ban = bitboard::BitBoard::from(rfen).unwrap();
        ban.put();
        let mut w = weight::Weight::new();
        w.init();
        let res_nosimde = w.evaluatev9bb(&bban);
        let res_simd = w.evaluatev9bb_simd(&bban);
        let res_simdavx = w.evaluatev9bb_simdavx(&bban);
        assert!(dbg_assert_eq(&res_nosimde, &res_simd));
        assert!(dbg_assert_eq(&res_nosimde, &res_simdavx));
        // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
    }
}

#[cfg(target_arch="aarch64")]
#[test]
fn testweight() {
    let rfens = [
        "h/H/h/H/h/H/h/H b",
        "h/H/h/H/h/H/h/H w",
        "H/h/H/h/H/h/H/h b",
        "H/h/H/h/H/h/H/h w",
        "h/H/8/H/h/H/h/H b",
        "h/H/h/8/h/H/h/H w",
        "H/h/H/h/8/h/H/h b",
        "H/h/H/h/H/8/H/h w",
        "aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa b",
        "aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA w",
        "8/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa b",
        "aAaAaAaA/AaAaAaAa/8/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/8/aAaAaAaA/AaAaAaAa/aAaAaAaA w",
        "AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/AaAaAaAa/aAaAaAaA/8/aAaAaAaA b",
        "aAaAaAaA/8/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA b",
        "aAaAaAaA/aAaAaAaA/aAaAaAaA/8/aAaAaAaA/aAaAaAaA/aAaAaAaA/aAaAaAaA w",
        "aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa/aA1AaAaA/Aa1aAaAa b",
        "1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa/1AaAaAaA/1aAaAaAa w",
        "AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1/AaAaAaA1/aAaAaAa1 w",
        "A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA/A1AaAaAa/a1aAaAaA b",
        "aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A/aAaAaA1A b",
        "aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA/aAaAa1aA w",
    ];
    for rfen in rfens.iter() {
        let bban = bitboard::BitBoard::from(rfen).unwrap();
        bban.put();
        let mut w = weight::Weight::new();
        w.init();
        let res_nosimdi = w.evaluatev9bb(&bban);
        let res_nosimdi_old = w.evaluatev9bb_old(&bban);
        let res_simdmul = w.evaluatev9bb_simd_mul(&bban);
        // let res_simd = w.evaluatev9bb_simd(&bban);
        assert!(dbg_assert_eq(&res_nosimdi, &res_nosimdi_old));
        assert!(dbg_assert_eq(&res_nosimdi, &res_simdmul));
        // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
    }
}
