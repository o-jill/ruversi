use super::*;
use rand::Rng;
use std::{fs, io::{BufReader, BufRead}};

#[cfg(target_arch="x86_64")]
use std::arch::x86_64;

#[cfg(target_arch="aarch64")]
use std::arch::aarch64::*;


/*
 * input: NUMCELL * NUMCELL + 1(teban) + 2(fixedstones) + 1
 * hidden: 8 + 1
 * output: 1
 */
const N_INPUT : usize = board::CELL_2D + 1 + 2;
const N_HIDDEN : usize = 128;
pub const N_HIDDEN2 : usize = 16;
const N_OUTPUT : usize = 1;
const N_WEIGHT_TEBAN : usize =  board::CELL_2D * N_HIDDEN;
const N_WEIGHT_FIXST_B : usize = N_WEIGHT_TEBAN + N_HIDDEN;
const N_WEIGHT_FIXST_W : usize = N_WEIGHT_FIXST_B + N_HIDDEN;
const N_WEIGHT_INPUTBIAS : usize = N_WEIGHT_FIXST_W + N_HIDDEN;
const N_WEIGHT_LAYER1 : usize = N_WEIGHT_INPUTBIAS + N_HIDDEN;
const N_WEIGHT_LAYER1BIAS : usize = N_WEIGHT_LAYER1 + N_HIDDEN * N_HIDDEN2;
const N_WEIGHT_LAYER2 : usize = N_WEIGHT_LAYER1BIAS + N_HIDDEN2;
const N_WEIGHT_LAYER2BIAS : usize = N_WEIGHT_LAYER2 + N_HIDDEN2;
const N_WEIGHT : usize =
  (N_INPUT + 1) * N_HIDDEN + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;

#[allow(dead_code)]
const WSZV1 : usize = (board::CELL_2D + 1 + 1) * 4 + 4 + 1;
#[allow(dead_code)]
const WSZV2 : usize = WSZV1;
const WSZV3 : usize = (board::CELL_2D + 1 + 2 + 1) * 4 + 4 + 1;
const WSZV4 : usize = (board::CELL_2D + 1 + 2 + 1) * 8 + 8 + 1;
const WSZV5 : usize = (board::CELL_2D + 1 + 2 + 1) * 16 + 16 + 1;
const WSZV6 : usize = (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN + N_HIDDEN + 1;
const WSZV7 : usize = (board::CELL_2D + 1 + 2 + 1) * 32
        + (32 + 1) * 16 + 16 + 1;
const WSZV8 : usize = (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN
        + (N_HIDDEN + 1) * N_HIDDEN2 + N_HIDDEN2 + 1;

const EXP_HI : f64 = 88.3762626647949;
const EXP_LO : f64 = -EXP_HI;

const CEPHES_LOG2EF : f64 = std::f64::consts::LOG2_E;
// const CEPHES_LOG2EF : f64 = 1.44269504088896341;
const CEPHES_EXP_C1 : f64 = 0.693359375;
const CEPHES_EXP_C2 : f64 = -2.12194440e-4;

const CEPHES_EXP_P0 : f64 = 1.9875691500E-4;
const CEPHES_EXP_P1 : f64 = 1.3981999507E-3;
const CEPHES_EXP_P2 : f64 = 8.3334519073E-3;
const CEPHES_EXP_P3 : f64 = 4.1665795894E-2;
const CEPHES_EXP_P4 : f64 = 1.6666665459E-1;
const CEPHES_EXP_P5 : f64 = 5.0000001201E-1;


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
}

impl EvalFile {
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

#[repr(align(32))]
pub struct Weight {
    pub weight : [f32 ; N_WEIGHT]
}

impl Default for Weight {
    fn default() -> Self {
        Self::new()
    }
}

impl Weight {
    pub fn new() -> Weight {
        Weight {
            weight: [0.0 ; N_WEIGHT]
        }
    }

    pub fn init(&mut self) {
        let mut rng = rand::thread_rng();
        let range =
            f64::sqrt(6.0) / f64::sqrt((N_INPUT + N_HIDDEN + N_OUTPUT) as f64);

        for a in self.weight.iter_mut() {
            *a = (rng.gen::<f64>() * 2.0 * range - range) as f32;
        }
    }

    /// fill zero.
    pub fn clear(&mut self) {
        self.weight.iter_mut().for_each(|m| *m = 0.0);
    }

    pub fn wban(&self) -> &[f32] {
        &self.weight[0..]
        // or &self.weight[0..N_WEIGHT_TEBAN]
    }

    pub fn wteban(&self) -> &[f32] {
        &self.weight[N_WEIGHT_TEBAN..N_WEIGHT_FIXST_W]
    }

    pub fn wfixedstones(&self) -> &[f32] {
      &self.weight[N_WEIGHT_FIXST_B..N_WEIGHT_INPUTBIAS]
    }

    pub fn wfixedstone_b(&self) -> &[f32] {
        &self.weight[N_WEIGHT_FIXST_B..N_WEIGHT_FIXST_W]
    }

    pub fn wfixedstone_w(&self) -> &[f32] {
        &self.weight[N_WEIGHT_FIXST_W..N_WEIGHT_INPUTBIAS]
    }

    pub fn wibias(&self) -> &[f32] {
        &self.weight[N_WEIGHT_INPUTBIAS..N_WEIGHT_LAYER1]
    }

    pub fn wlayer1(&self) -> &[f32] {
        &self.weight[N_WEIGHT_LAYER1..N_WEIGHT_LAYER1BIAS]
    }

    pub fn wl1bias(&self) -> &[f32] {
        &self.weight[N_WEIGHT_LAYER1BIAS..N_WEIGHT_LAYER2]
    }

    pub fn wlayer2(&self) -> &[f32] {
        &self.weight[N_WEIGHT_LAYER2..N_WEIGHT_LAYER2BIAS]
    }

    pub fn wl2bias(&self) -> f32 {
        *self.weight.last().unwrap()
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
        if file.is_err() {return Err(file.err().unwrap().to_string());}

        let file = file.unwrap();
        let lines = BufReader::new(file);
        for line in lines.lines() {
            match line {
                Ok(l) => {
                    if l.starts_with("#") {
                        if format != EvalFile::Unknown {
                            continue;
                        }
                        let res = EvalFile::from(&l);
                        if res.is_some() {
                            format = res.unwrap();
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
        self.weight.copy_from_slice(&newtable);
        // println!("v8:{:?}", self.weight);
        Ok(())
    }

    fn write(f : &mut File, w : &[f32], ver : &EvalFile) {
        let sv = w.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        f.write_all(format!("{}\n", ver.to_str()).as_bytes()).unwrap();
        f.write_all(sv.join(",").as_bytes()).unwrap();
    }

    pub fn writev8(&self, path : &str) {
        let mut f = fs::File::create(path).unwrap();
        Weight::write(&mut f, &self.weight, &EvalFile::V8);
    }

    pub fn copy(&mut self, src : &Weight) {
        self.weight.copy_from_slice(&src.weight);
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x` - [4 ; f32]  
     * * `y` - [exp(-x[0]), exp(-x[1]), exp(-x[2]), exp(-x[3])]  
     */
    #[cfg(target_arch="x86_64")]
    fn expmx_ps( x : *const f32, y : *mut f32) {
        unsafe {
            let x4 = x86_64::_mm_load_ps(x);
            let y4 = Weight::expmx_ps_simd(x4);
            x86_64::_mm_store_ps(y, y4);
        }
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x` - [4 ; f32]  
     * * `y` - [exp(-x[0]), exp(-x[1]), exp(-x[2]), exp(-x[3])]  
     */
    #[cfg(target_arch="aarch64")]
    fn expmx_ps( x : *const f32, y : *mut f32) {
        unsafe {
            let x4 = vld1q_f32(x);
            let y4 = Weight::expmx_ps_simd(x4);
            vst1q_f32(y, y4);
        }
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x4` - [4 ; f32] as x86_64::__m128
     * 
     * # Return value  
     * [exp(-x4[0]), exp(-x4[1]), exp(-x4[2]), exp(-x4[3])] as x86_64::__m128.
     */
    #[cfg(target_arch="x86_64")]
    unsafe fn expmx_ps_simd(x4 : x86_64::__m128) -> x86_64::__m128 {
        // let x4 = x86_64::_mm_load_ps(x);
        // clip x
        let max4 = x86_64::_mm_set1_ps(EXP_HI as f32);
        let x4 = x86_64::_mm_min_ps(x4, max4);
        let min4 = x86_64::_mm_set1_ps(EXP_LO as f32);
        let x4 = x86_64::_mm_max_ps(x4, min4);
        let m1 = x86_64::_mm_set1_ps(-1.0);
        let x4 = x86_64::_mm_mul_ps(x4, m1);

        /* express exp(x) as exp(g + n*log(2)) */
        let log2ef = x86_64::_mm_set1_ps(CEPHES_LOG2EF as f32);
        let fx = x86_64::_mm_mul_ps(x4, log2ef);
        let zp5 = x86_64::_mm_set1_ps(CEPHES_EXP_P5 as f32);
        let fx = x86_64::_mm_add_ps(fx, zp5);
        let emm0 = x86_64::_mm_cvtps_epi32(fx);
        let tmp = x86_64::_mm_cvtepi32_ps(emm0);

        let mask = x86_64::_mm_cmpgt_ps(tmp, fx);
        let one = x86_64::_mm_set1_ps(1.0);
        let mask = x86_64::_mm_and_ps(mask, one);
        let fx = x86_64::_mm_sub_ps(tmp, mask);

        let c1 = x86_64::_mm_set1_ps(CEPHES_EXP_C1 as f32);
        let tmp = x86_64::_mm_mul_ps(fx, c1);
        let c2 = x86_64::_mm_set1_ps(CEPHES_EXP_C2 as f32);
        let z4 = x86_64::_mm_mul_ps(fx, c2);
        let x4 = x86_64::_mm_sub_ps(x4, tmp);
        let x4 = x86_64::_mm_sub_ps(x4, z4);

        let z4 = x86_64::_mm_mul_ps(x4, x4);

        let y4 = x86_64::_mm_set1_ps(CEPHES_EXP_P0 as f32);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p1 = x86_64::_mm_set1_ps(CEPHES_EXP_P1 as f32);
        let y4 = x86_64::_mm_add_ps(y4, exp_p1);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p2 = x86_64::_mm_set1_ps(CEPHES_EXP_P2 as f32);
        let y4 = x86_64::_mm_add_ps(y4, exp_p2);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p3 = x86_64::_mm_set1_ps(CEPHES_EXP_P3 as f32);
        let y4 = x86_64::_mm_add_ps(y4, exp_p3);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p4 = x86_64::_mm_set1_ps(CEPHES_EXP_P4 as f32);
        let y4 = x86_64::_mm_add_ps(y4, exp_p4);
        let y4 = x86_64::_mm_mul_ps(y4, x4);
        let exp_p5 = x86_64::_mm_set1_ps(CEPHES_EXP_P5 as f32);
        let y4 = x86_64::_mm_add_ps(y4, exp_p5);
        let y4 = x86_64::_mm_mul_ps(y4, z4);
        let y4 = x86_64::_mm_add_ps(y4, x4);
        let y4 = x86_64::_mm_add_ps(y4, one);

        let emm0 = x86_64::_mm_cvttps_epi32(fx);
        let _pi32_0x7f = x86_64::_mm_set1_epi32(0x7f);
        let emm0 = x86_64::_mm_add_epi32(emm0, _pi32_0x7f);
        let emm0 = x86_64::_mm_slli_epi32(emm0, 23);
        let pow2n = x86_64::_mm_castsi128_ps(emm0);

        let y4 = x86_64::_mm_mul_ps(y4, pow2n);
        y4
        // x86_64::_mm_store_ps(y, y4);
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x4` - [4 ; f32] as x86_64::__m128
     * 
     * # Return value  
     * [exp(-x4[0]), exp(-x4[1]), exp(-x4[2]), exp(-x4[3])] as x86_64::__m128.
     */
    #[cfg(target_arch="aarch64")]
    unsafe fn expmx_ps_simd(x4 : float32x4_t) -> float32x4_t {
        // clip x
        let max4 = vmovq_n_f32(EXP_HI as f32);
        let x4 = vminq_f32(x4, max4);
        let min4 = vmovq_n_f32(EXP_LO as f32);
        let x4 = vmaxq_f32(x4, min4);
        let m1 = vmovq_n_f32(-1.0);
        let x4 = vmulq_f32(x4, m1);

        /* express exp(x) as exp(g + n*log(2)) */
        let log2ef = vmovq_n_f32(CEPHES_LOG2EF as f32);
        let zp5 = vmovq_n_f32(CEPHES_EXP_P5 as f32);
        let fx = vmlaq_f32(zp5, x4, log2ef);
        let emm0 = vcvtq_s32_f32(fx);
        let tmp = vcvtq_f32_s32(emm0);

        let mask = vcgtq_f32(tmp, fx);
        let one = vmovq_n_f32(1.0);
        let mask = vreinterpretq_f32_u32(vandq_u32(
                mask, vreinterpretq_u32_f32(one)));
        let fx = vsubq_f32(tmp, mask);

        let c1 = vmovq_n_f32(CEPHES_EXP_C1 as f32);
        let tmp = vmulq_f32(fx, c1);
        let c2 = vmovq_n_f32(CEPHES_EXP_C2 as f32);
        let z4 = vmulq_f32(fx, c2);
        let x4 = vsubq_f32(x4, tmp);
        let x4 = vsubq_f32(x4, z4);

        let z4 = vmulq_f32(x4, x4);

        let y4 = vmovq_n_f32(CEPHES_EXP_P0 as f32);
        let exp_p1 = vmovq_n_f32(CEPHES_EXP_P1 as f32);
        let y4 = vmlaq_f32(exp_p1, y4, x4);
        let exp_p2 = vmovq_n_f32(CEPHES_EXP_P2 as f32);
        let y4 = vmlaq_f32(exp_p2, y4, x4);
        let exp_p3 = vmovq_n_f32(CEPHES_EXP_P3 as f32);
        let y4 = vmlaq_f32(exp_p3, y4, x4);
        let exp_p4 = vmovq_n_f32(CEPHES_EXP_P4 as f32);
        let y4 = vmlaq_f32(exp_p4, y4, x4);
        let exp_p5 = vmovq_n_f32(CEPHES_EXP_P5 as f32);
        let y4 = vmlaq_f32(exp_p5, y4, x4);
        let y4 = vmlaq_f32(x4, y4, z4);
        let y4 = vaddq_f32(y4, one);

        let emm0 = vcvtq_s32_f32(fx);
        let _pi32_0x7f = vmovq_n_s32(0x7f);
        let emm0 = vaddq_s32(emm0, _pi32_0x7f);
        let emm0 = vshlq_n_s32(emm0, 23);
        let pow2n = vreinterpretq_f32_s32(emm0);

        vmulq_f32(y4, pow2n)
    }

    #[inline]
    #[cfg(target_arch="aarch64")]
    unsafe fn expmx_ps_simdx2(x41 : float32x4_t, x42 : float32x4_t)
            -> (float32x4_t, float32x4_t) {
        // clip x
        let max4 = vmovq_n_f32(EXP_HI as f32);
        let x41 = vminq_f32(x41, max4);
        let x42 = vminq_f32(x42, max4);
        let min4 = vmovq_n_f32(EXP_LO as f32);
        let x41 = vmaxq_f32(x41, min4);
        let x42 = vmaxq_f32(x42, min4);
        let m1 = vmovq_n_f32(-1.0);
        let x41 = vmulq_f32(x41, m1);
        let x42 = vmulq_f32(x42, m1);

        /* express exp(x) as exp(g + n*log(2)) */
        let log2ef = vmovq_n_f32(CEPHES_LOG2EF as f32);
        let zp5 = vmovq_n_f32(CEPHES_EXP_P5 as f32);
        let fx1 = vmlaq_f32(zp5, x41, log2ef);
        let fx2 = vmlaq_f32(zp5, x42, log2ef);
        let emm01 = vcvtq_s32_f32(fx1);
        let emm02 = vcvtq_s32_f32(fx2);
        let tmp1 = vcvtq_f32_s32(emm01);
        let tmp2 = vcvtq_f32_s32(emm02);

        let mask1 = vcgtq_f32(tmp1, fx2);
        let mask2 = vcgtq_f32(tmp2, fx2);
        let one = vmovq_n_f32(1.0);
        let mask1 = vreinterpretq_f32_u32(vandq_u32(
                mask1, vreinterpretq_u32_f32(one)));
        let mask2 = vreinterpretq_f32_u32(vandq_u32(
                mask2, vreinterpretq_u32_f32(one)));
        let fx1 = vsubq_f32(tmp1, mask1);
        let fx2 = vsubq_f32(tmp2, mask2);

        let c1 = vmovq_n_f32(CEPHES_EXP_C1 as f32);
        let tmp1 = vmulq_f32(fx1, c1);
        let tmp2 = vmulq_f32(fx2, c1);
        let c2 = vmovq_n_f32(CEPHES_EXP_C2 as f32);
        let z41 = vmulq_f32(fx1, c2);
        let z42 = vmulq_f32(fx2, c2);
        let x41 = vsubq_f32(x41, tmp1);
        let x42 = vsubq_f32(x42, tmp2);
        let x41 = vsubq_f32(x41, z41);
        let x42 = vsubq_f32(x42, z42);

        let z41 = vmulq_f32(x41, x41);
        let z42 = vmulq_f32(x42, x42);

        let y4 = vmovq_n_f32(CEPHES_EXP_P0 as f32);
        let exp_p1 = vmovq_n_f32(CEPHES_EXP_P1 as f32);
        let y41 = vmlaq_f32(exp_p1, y4, x41);
        let y42 = vmlaq_f32(exp_p1, y4, x42);
        let exp_p2 = vmovq_n_f32(CEPHES_EXP_P2 as f32);
        let y41 = vmlaq_f32(exp_p2, y41, x41);
        let y42 = vmlaq_f32(exp_p2, y42, x42);
        let exp_p3 = vmovq_n_f32(CEPHES_EXP_P3 as f32);
        let y41 = vmlaq_f32(exp_p3, y41, x41);
        let y42 = vmlaq_f32(exp_p3, y42, x42);
        let exp_p4 = vmovq_n_f32(CEPHES_EXP_P4 as f32);
        let y41 = vmlaq_f32(exp_p4, y41, x41);
        let y42 = vmlaq_f32(exp_p4, y42, x42);
        let exp_p5 = vmovq_n_f32(CEPHES_EXP_P5 as f32);
        let y41 = vmlaq_f32(exp_p5, y41, x41);
        let y42 = vmlaq_f32(exp_p5, y42, x42);
        let y41 = vmlaq_f32(x41, y41, z41);
        let y42 = vmlaq_f32(x42, y42, z42);
        let y41 = vaddq_f32(y41, one);
        let y42 = vaddq_f32(y42, one);

        let emm01 = vcvtq_s32_f32(fx1);
        let emm02 = vcvtq_s32_f32(fx2);
        let _pi32_0x7f = vmovq_n_s32(0x7f);
        let emm01 = vaddq_s32(emm01, _pi32_0x7f);
        let emm02 = vaddq_s32(emm02, _pi32_0x7f);
        let emm01 = vshlq_n_s32(emm01, 23);
        let emm02 = vshlq_n_s32(emm02, 23);
        let pow2n1 = vreinterpretq_f32_s32(emm01);
        let pow2n2 = vreinterpretq_f32_s32(emm02);
        (vmulq_f32(y41, pow2n1), vmulq_f32(y42, pow2n2))
    }

    /**
     * exp(-x)
     * 
     * # Arguments  
     * * `x4` - [4 ; f32] as x86_64::__m128
     * 
     * # Return value  
     * [exp(-x4[0]), exp(-x4[1]), exp(-x4[2]), exp(-x4[3])] as x86_64::__m128.
     */
    #[cfg(target_arch="x86_64")]
    unsafe fn expmx_ps_simd256(x4 : x86_64::__m256) -> x86_64::__m256 {
        // let x4 = x86_64::_mm_load_ps(x);
        // clip x
        let max4 = x86_64::_mm256_set1_ps(EXP_HI as f32);
        let x4 = x86_64::_mm256_min_ps(x4, max4);
        let min4 = x86_64::_mm256_set1_ps(EXP_LO as f32);
        let x4 = x86_64::_mm256_max_ps(x4, min4);
        let m1 = x86_64::_mm256_set1_ps(-1.0);
        let x4 = x86_64::_mm256_mul_ps(x4, m1);

        /* express exp(x) as exp(g + n*log(2)) */
        let log2ef = x86_64::_mm256_set1_ps(CEPHES_LOG2EF as f32);
        let fx = x86_64::_mm256_mul_ps(x4, log2ef);
        let zp5 = x86_64::_mm256_set1_ps(CEPHES_EXP_P5 as f32);
        let fx = x86_64::_mm256_add_ps(fx, zp5);
        let emm0 = x86_64::_mm256_cvtps_epi32(fx);
        let tmp = x86_64::_mm256_cvtepi32_ps(emm0);

        let mask = x86_64::_mm256_cmp_ps(tmp, fx, x86_64::_CMP_GT_OS);
        let one = x86_64::_mm256_set1_ps(1.0);
        let mask = x86_64::_mm256_and_ps(mask, one);
        let fx = x86_64::_mm256_sub_ps(tmp, mask);

        let c1 = x86_64::_mm256_set1_ps(CEPHES_EXP_C1 as f32);
        let tmp = x86_64::_mm256_mul_ps(fx, c1);
        let c2 = x86_64::_mm256_set1_ps(CEPHES_EXP_C2 as f32);
        let z4 = x86_64::_mm256_mul_ps(fx, c2);
        let x4 = x86_64::_mm256_sub_ps(x4, tmp);
        let x4 = x86_64::_mm256_sub_ps(x4, z4);

        let z4 = x86_64::_mm256_mul_ps(x4, x4);

        let y4 = x86_64::_mm256_set1_ps(CEPHES_EXP_P0 as f32);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p1 = x86_64::_mm256_set1_ps(CEPHES_EXP_P1 as f32);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p1);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p2 = x86_64::_mm256_set1_ps(CEPHES_EXP_P2 as f32);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p2);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p3 = x86_64::_mm256_set1_ps(CEPHES_EXP_P3 as f32);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p3);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p4 = x86_64::_mm256_set1_ps(CEPHES_EXP_P4 as f32);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p4);
        let y4 = x86_64::_mm256_mul_ps(y4, x4);
        let exp_p5 = x86_64::_mm256_set1_ps(CEPHES_EXP_P5 as f32);
        let y4 = x86_64::_mm256_add_ps(y4, exp_p5);
        let y4 = x86_64::_mm256_mul_ps(y4, z4);
        let y4 = x86_64::_mm256_add_ps(y4, x4);
        let y4 = x86_64::_mm256_add_ps(y4, one);

        let emm0 = x86_64::_mm256_cvttps_epi32(fx);
        let _pi32_0x7f = x86_64::_mm256_set1_epi32(0x7f);
        let emm0 = x86_64::_mm256_add_epi32(emm0, _pi32_0x7f);
        let emm0 = x86_64::_mm256_slli_epi32(emm0, 23);
        let pow2n = x86_64::_mm256_castsi256_ps(emm0);

        let y4 = x86_64::_mm256_mul_ps(y4, pow2n);
        y4
        // x86_64::_mm_store_ps(y, y4);
    }

    pub fn evaluatev7(&self, ban : &board::Board) -> f32 {
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        
        let fs = ban.fixedstones();
        
        let mut sum = self.wl2bias();
        
        let ow = self.wban();
        let wtbn = self.wteban();
        let wfs = self.wfixedstones();
        let wdc = self.wibias();
        let wh = self.wlayer1();
        let whdc = self.wl1bias();
        let wh2 = self.wlayer2();
        let mut hid = [0f32 ; N_HIDDEN];
        for i in 0..N_HIDDEN {
            let w1 = &ow[i * board::CELL_2D .. (i + 1) * board::CELL_2D];
            let mut hidsum : f32 = wdc[i];
            for (idx, c)  in cells.iter().enumerate() {
                hidsum += *c as f32 * w1[idx];
            }
            hidsum += teban * wtbn[i];
            hidsum += wfs[i] * fs.0 as f32;
            hidsum += wfs[i + N_HIDDEN] * fs.1 as f32;
            hid[i] = 1.0 / (f32::exp(-hidsum) + 1.0);
        }
        for j in 0..N_HIDDEN2 {
            let whd = &wh[j * N_HIDDEN .. j * N_HIDDEN + N_HIDDEN];
            let mut hidsum : f32 = whdc[j];
            for (idx, c)  in hid.iter().enumerate() {
                hidsum += *c * whd[idx];
            }
            sum += wh2[j] / (f32::exp(-hidsum) + 1.0);
        }
        sum
    }

    pub fn evaluatev7bb(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wban();
        let wtbn = self.wteban();
        let wfs = self.wfixedstones();
        let wdc = self.wibias();
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
            // sigmoid
            // *h = 1f32 / ((-hidsum).exp() + 1.0);
            // relu
            *h = if hidsum > 0.0 {hidsum} else {0.0};
        }

        let mut sum = self.wl2bias();
        let wh = self.wlayer1();
        let whdc = self.wl1bias();
        let wh2 = self.wlayer2();
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = whdc[i];
            for (j, h1) in hid.iter().enumerate() {
                hidsum2 = h1.mul_add(wh[j + i * N_HIDDEN], hidsum2);
                // hidsum2 += h1 * wh[j + i * N_HIDDEN];
            }
            // sigmoid
            // sum += wh2[i] / ((-hidsum2).exp() + 1f32)
            // relu
            sum += if hidsum2 > 0.0 {wh2[i] * hidsum2} else {0.0};
        }
        sum
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev3_simd(&self, ban : &board::Board) -> f32 {
        let cells = &ban.cells;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];

        const N : usize = 4;
        let mut hidsum : [f32 ; N] = [0.0f32 ; N];
        let mut emx : [f32 ; N] = [0.0f32 ; N];
        let mut sumarr : [f32 ; N] = [0.0f32 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum44 : [f32 ; N * N] = [0.0f32 ; N * N];

            for n in 0..N {
                let res4 = sum44[n * N..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                let mut sum4: x86_64::__m128;
                unsafe {
                    sum4 = x86_64::_mm_setzero_ps();
                }
                const M : usize = 16;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    unsafe {
                        let x41 = x86_64::_mm_load_ps(w1[idx..].as_ptr());
                        let x42 = x86_64::_mm_load_ps(w1[idx + 4..].as_ptr());
                        let x43 = x86_64::_mm_load_ps(w1[idx + 8..].as_ptr());
                        let x44 = x86_64::_mm_load_ps(w1[idx + 12..].as_ptr());

                        let c8 = x86_64::_mm_load_si128(cells[idx..].as_ptr() as *const x86_64::__m128i);
                        let zero = x86_64::_mm_setzero_si128();
                        // to i16
                        let s16 = x86_64::_mm_cmpgt_epi8(zero, c8);
                        let c4l = x86_64::_mm_unpacklo_epi8(c8, s16);
                        let c4h = x86_64::_mm_unpackhi_epi8(c8, s16);
                        // to i32
                        let s4l = x86_64::_mm_cmpgt_epi16(zero, c4l);
                        let s4h = x86_64::_mm_cmpgt_epi16(zero, c4h);
                        let c41 = x86_64::_mm_unpacklo_epi16(c4l, s4l);
                        let c42 = x86_64::_mm_unpackhi_epi16(c4l, s4l);
                        let c43 = x86_64::_mm_unpacklo_epi16(c4h, s4h);
                        let c44 = x86_64::_mm_unpackhi_epi16(c4h, s4h);

                        let f41 = x86_64::_mm_cvtepi32_ps(c41);
                        let f42 = x86_64::_mm_cvtepi32_ps(c42);
                        let f43 = x86_64::_mm_cvtepi32_ps(c43);
                        let f44 = x86_64::_mm_cvtepi32_ps(c44);

                        let mul1 = x86_64::_mm_mul_ps(x41, f41);
                        let mul2 = x86_64::_mm_mul_ps(x42, f42);
                        let mul3 = x86_64::_mm_mul_ps(x43, f43);
                        let mul4 = x86_64::_mm_mul_ps(x44, f44);

                        let sum12 = x86_64::_mm_add_ps(mul1, mul2);
                        let sum34 = x86_64::_mm_add_ps(mul3, mul4);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                    }
                }
                unsafe {
                    x86_64::_mm_store_ps(res4, sum4);
                }
            }

            unsafe {
                let mut x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let mut x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let mut x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let mut x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);
                // teban
                let wtbn = x86_64::_mm_load_ps(wtbn[hidx..].as_ptr());
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // fixed stones
                let wfsb4 = x86_64::_mm_load_ps(wfs[hidx..].as_ptr());
                let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm_mul_ps(wfsb4, fsb);
                let wfsw4 = x86_64::_mm_load_ps(wfs[hidx + N_HIDDEN..].as_ptr());
                let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm_mul_ps(wfsw4, fsw);
                let fsbw = x86_64::_mm_add_ps(fsb4, fsw4);
                let h1234 = x86_64::_mm_add_ps(h1234, fsbw);
                // dc
                let wdc4 = x86_64::_mm_load_ps(wdc[hidx..].as_ptr());
                let h1234 = x86_64::_mm_add_ps(h1234, wdc4);
                x86_64::_mm_store_ps(hidsum.as_mut_ptr(), h1234);
            }
            Weight::expmx_ps(hidsum.as_ptr(), emx.as_mut_ptr());
            unsafe {
                let emx4 = x86_64::_mm_load_ps(emx.as_ptr());
                let one = x86_64::_mm_set1_ps(1.0);
                let hsp14 = x86_64::_mm_add_ps(emx4, one);
                let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());

                let y4 = x86_64::_mm_div_ps(wh4, hsp14);
                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
            }
            // for s in sumarr {
            //     sum += s;
            // }
            sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        sum
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev3bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];

        const N : usize = 4;
        let mut sumarr : [f32 ; N] = [0.0f32 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum44 : [f32 ; N * N] = [0.0f32 ; N * N];

            for n in 0..N {
                let res4 = sum44[n * N..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum4: x86_64::__m128;
                unsafe {
                    sum4 = x86_64::_mm_setzero_ps();
                }
                const M : usize = 16;
                let mut bit8 : u64 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let b81 = (bit8 & black) >> (2 * j);
                    let w81 = (bit8 & white) >> (2 * j);
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> (2 * j + 1);
                    let w82 = (bit8 & white) >> (2 * j + 1);
                    bit8 <<= 1;

                    unsafe {
                        let b08 = x86_64::_mm_set_epi64x(b82 as i64, b81 as i64);
                        let w08 = x86_64::_mm_set_epi64x(w82 as i64, w81 as i64);
                        let one = x86_64::_mm_set1_epi8(1);
                        let bm08 = x86_64::_mm_cmpeq_epi8(b08, one);
                        let wm08 = x86_64::_mm_cmpeq_epi8(w08, one);
                        let b16l = x86_64::_mm_unpacklo_epi8(bm08, bm08);
                        let b16h = x86_64::_mm_unpackhi_epi8(bm08, bm08);
                        let w16l = x86_64::_mm_unpacklo_epi8(wm08, wm08);
                        let w16h = x86_64::_mm_unpackhi_epi8(wm08, wm08);
                        let bm1 = x86_64::_mm_unpacklo_epi16(b16l, b16l);
                        let bm2 = x86_64::_mm_unpackhi_epi16(b16l, b16l);
                        let bm3 = x86_64::_mm_unpacklo_epi16(b16h, b16h);
                        let bm4 = x86_64::_mm_unpackhi_epi16(b16h, b16h);
                        let wm1 = x86_64::_mm_unpacklo_epi16(w16l, w16l);
                        let wm2 = x86_64::_mm_unpackhi_epi16(w16l, w16l);
                        let wm3 = x86_64::_mm_unpacklo_epi16(w16h, w16h);
                        let wm4 = x86_64::_mm_unpackhi_epi16(w16h, w16h);
                        let ex1 = x86_64::_mm_or_si128(bm1, wm1);
                        let ex2 = x86_64::_mm_or_si128(bm2, wm2);
                        let ex3 = x86_64::_mm_or_si128(bm3, wm3);
                        let ex4 = x86_64::_mm_or_si128(bm4, wm4);
                        let x41 = x86_64::_mm_load_ps(w1.as_ptr().add(idx));
                        let x42 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 4));
                        let x43 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 8));
                        let x44 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 12));
                        let minus = x86_64::_mm_set1_ps(-0.0);
                        let mn1 = x86_64::_mm_and_ps(
                                x86_64::_mm_castsi128_ps(wm1), minus);
                        let mn2 = x86_64::_mm_and_ps(
                                x86_64::_mm_castsi128_ps(wm2), minus);
                        let mn3 = x86_64::_mm_and_ps(
                                x86_64::_mm_castsi128_ps(wm3), minus);
                        let mn4 = x86_64::_mm_and_ps(
                                x86_64::_mm_castsi128_ps(wm4), minus);
                        let m41 = x86_64::_mm_xor_ps(x41, mn1);
                        let m42 = x86_64::_mm_xor_ps(x42, mn2);
                        let m43 = x86_64::_mm_xor_ps(x43, mn3);
                        let m44 = x86_64::_mm_xor_ps(x44, mn4);
                        let w1 = x86_64::_mm_and_ps(
                                m41, x86_64::_mm_castsi128_ps(ex1));
                        let w2 = x86_64::_mm_and_ps(
                                m42, x86_64::_mm_castsi128_ps(ex2));
                        let w3 = x86_64::_mm_and_ps(
                                m43, x86_64::_mm_castsi128_ps(ex3));
                        let w4 = x86_64::_mm_and_ps(
                                m44, x86_64::_mm_castsi128_ps(ex4));

                        let sum12 = x86_64::_mm_add_ps(w1, w2);
                        let sum34 = x86_64::_mm_add_ps(w3, w4);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                    }
                }
                unsafe {
                    x86_64::_mm_store_ps(res4, sum4);
                }
            }

            unsafe {
                let mut x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let mut x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let mut x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let mut x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);
                // teban
                let wtbn = x86_64::_mm_load_ps(wtbn[hidx..].as_ptr());
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // fixed stones
                let wfsb4 = x86_64::_mm_load_ps(wfs[hidx..].as_ptr());
                let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm_mul_ps(wfsb4, fsb);
                let wfsw4 = x86_64::_mm_load_ps(wfs[hidx + N_HIDDEN..].as_ptr());
                let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm_mul_ps(wfsw4, fsw);
                let fsbw = x86_64::_mm_add_ps(fsb4, fsw4);
                let h1234 = x86_64::_mm_add_ps(h1234, fsbw);
                // dc
                let wdc4 = x86_64::_mm_load_ps(wdc[hidx..].as_ptr());
                let h1234 = x86_64::_mm_add_ps(h1234, wdc4);

                let emx4 = Weight::expmx_ps_simd(h1234);
                let one = x86_64::_mm_set1_ps(1.0);
                let hsp14 = x86_64::_mm_add_ps(emx4, one);
                let wh4 = x86_64::_mm_load_ps(wh[hidx..].as_ptr());

                let y4 = x86_64::_mm_div_ps(wh4, hsp14);
                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
            }
            // for s in sumarr {
            //     sum += s;
            // }
            sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        sum
    }

    #[cfg(target_arch="aarch64")]
    pub fn evaluatev3bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        use std::arch::aarch64::*;
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let (fsb, fsw) = ban.fixedstones();

        let mut res = *ow.last().unwrap();

        let wtbn = &ow[bitboard::CELL_2D * N_HIDDEN .. ];
        let wfs = &ow[(bitboard::CELL_2D + 1) * N_HIDDEN .. ];
        let wdc = &ow[(bitboard::CELL_2D + 1 + 2) * N_HIDDEN .. ];
        let wh = &ow[(bitboard::CELL_2D + 1 + 2 + 1) * N_HIDDEN .. ];
        const N : usize = 8;

        for i in (0..N_HIDDEN).step_by(N) {
            let mut sumn = [0.0f32 ; N];

            for n in 0..N {
                let w1 = &ow[(i + n) * bitboard::CELL_2D .. ];
                const M : usize = 16;
                let bit8 = 0x0101010101010101u64;
                for j in 0..bitboard::CELL_2D / M {
                    let idx = j * M;
                    let b81 = bit8 & (black >> (j * 2));
                    let w81 = bit8 & (white >> (j * 2));
                    let b82 = bit8 & (black >> (j * 2 + 1));
                    let w82 = bit8 & (white >> (j * 2 + 1));

                    unsafe {
                        let b08 = vmov_n_u64(b81 * 0xffu64);
                        let b082 = vmov_n_u64(b82 * 0xffu64);
                        let w08 = vmov_n_u64(w81 * 0xffu64);
                        let w082 = vmov_n_u64(w82 * 0xffu64);
                        let b04 = vmovl_s8(vreinterpret_s8_u64(b08));
                        let b042 = vmovl_s8(vreinterpret_s8_u64(b082));
                        let w04 = vmovl_s8(vreinterpret_s8_u64(w08));
                        let w042 = vmovl_s8(vreinterpret_s8_u64(w082));
                        let b02 = vmovl_s16(vget_low_s16(b04));
                        let b022 = vmovl_s16(vget_low_s16(b042));
                        let b12 = vmovl_high_s16(b04);
                        let b122 = vmovl_high_s16(b042);
                        let w02 = vmovl_s16(vget_low_s16(w04));
                        let w022 = vmovl_s16(vget_low_s16(w042));
                        let w12 = vmovl_high_s16(w04);
                        let w122 = vmovl_high_s16(w042);
                        let ex1 = vorrq_s32(b02, w02);
                        let ex12 = vorrq_s32(b022, w022);
                        let ex2 = vorrq_s32(b12, w12);
                        let ex22 = vorrq_s32(b122, w122);
                        let minus = vreinterpretq_s32_f32(vmovq_n_f32(-0.0));
                        let mn1 = vandq_s32(minus, w02);
                        let mn12 = vandq_s32(minus, w022);
                        let mn2 = vandq_s32(minus, w12);
                        let mn22 = vandq_s32(minus, w122);
                        let w41 = vld1q_f32_x4(w1.as_ptr().add(idx));
                        let w1 = veorq_s32(mn1, vreinterpretq_s32_f32(w41.0));
                        let w12 = veorq_s32(mn12, vreinterpretq_s32_f32(w41.2));
                        let w2 = veorq_s32(mn2, vreinterpretq_s32_f32(w41.1));
                        let w22 = veorq_s32(mn22, vreinterpretq_s32_f32(w41.3));
                        let w1 = vandq_s32(ex1, w1);
                        let w12 = vandq_s32(ex12, w12);
                        let w2 = vandq_s32(ex2, w2);
                        let w22 = vandq_s32(ex22, w22);
                        let sum = vaddq_f32(vreinterpretq_f32_s32(w1),
                                                        vreinterpretq_f32_s32(w2));
                        let sum2 = vaddq_f32(vreinterpretq_f32_s32(w12),
                                                        vreinterpretq_f32_s32(w22));
                        let sum = vaddvq_f32(vaddq_f32(sum, sum2));
                        sumn[n] += sum;
                    }
                }
            }
            unsafe {
                let sum4 = vld1q_f32_x2(sumn.as_ptr());

                let tbn = vmovq_n_f32(teban);
                let wtb = vld1q_f32_x2(wtbn.as_ptr().add(i));
                let sum41 = vmlaq_f32(sum4.0, tbn, wtb.0);
                let sum42 = vmlaq_f32(sum4.1, tbn, wtb.1);

                let fsb4 = vmovq_n_f32(fsb as f32);
                let wfsb = vld1q_f32_x2(wfs.as_ptr().add(i));
                let sum4 = vmlaq_f32(sum41, fsb4, wfsb.0);
                let sum42 = vmlaq_f32(sum42, fsb4, wfsb.1);

                let fsw4 = vmovq_n_f32(fsw as f32);
                let wfsw = vld1q_f32_x2(wfs.as_ptr().add(i + N_HIDDEN));
                let sum4 = vmlaq_f32(sum4, fsw4, wfsw.0);
                let sum42 = vmlaq_f32(sum42, fsw4, wfsw.1);

                let wdc4 = vld1q_f32_x2(wdc.as_ptr().add(i));
                let sum4 = vaddq_f32(sum4, wdc4.0);
                let sum42 = vaddq_f32(sum42, wdc4.1);
                // vst1q_f32(sumn.as_mut_ptr(), sum4);
                // vst1q_f32(sumn.as_mut_ptr().add(4), sum42);

                let (expmx1, expmx2) = Self::expmx_ps_simdx2(sum4, sum42);
                let one = vmovq_n_f32(1.0);
                let expmx1 = vaddq_f32(expmx1, one);
                let expmx2 = vaddq_f32(expmx2, one);
                let wh8 = vld1q_f32_x2(wh.as_ptr().add(i));
                let h1 = vdivq_f32(wh8.0, expmx1);
                let h2 = vdivq_f32(wh8.1, expmx2);
                res += vaddvq_f32(vaddq_f32(h1, h2));
                // 1860nps

                // let expmx = Self::expmx_ps_simd(sum4);
                // let expmx1 = vaddq_f32(expmx, vmovq_n_f32(1.0));
                // let remx = vrecpeq_f32(expmx1);
                // let wh4 = vld1q_f32(wh.as_ptr().add(i));
                // res += vaddvq_f32(vmulq_f32(remx, wh4));
                // expmx_ps_simd is slower than exp()x4 on M2 ...
                // 1950nps
            }
            // for n in 0 .. N {
            //     // sumn[n] = (-sumn[n]).exp();
            //     res += wh[i + n] / ((-sumn[n]).exp() + 1.0);
            // }  // 1820nps
            // unsafe {
            //     let expmx4 = vld1q_f32_x2(sumn.as_ptr());
            //     // let expmx = vld1q_f32(sumn.as_ptr());
            //     // let expmx2 = vld1q_f32(sumn.as_ptr().add(4));
            //     let one = vmovq_n_f32(1.0);
            //     let expmx1 = vaddq_f32(expmx4.0, one);
            //     let expmx12 = vaddq_f32(expmx4.1, one);
            //     let wh4 = vld1q_f32_x2(wh.as_ptr().add(i));
            //     // let wh4 = vld1q_f32(wh.as_ptr().add(i));
            //     // let wh42 = vld1q_f32(wh.as_ptr().add(i + 4));
            //     res += vaddvq_f32(vaddq_f32(vdivq_f32(wh4.0, expmx1),
            //                              vdivq_f32(wh4.1, expmx12)));
            // }
        }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev7bb_simd(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wban();
        let wtbn = self.wteban();
        let wfs = self.wfixedstones();
        let wdc = self.wibias();

        let mut hid = [0f32 ; N_HIDDEN];
        const N : usize = 8;

        for i in (0..N_HIDDEN).step_by(N) {
            let mut sum44 : [f32 ; N * 4] = [0f32 ; N * 4];

            for n in 0..N {
                let res4 = sum44[n * 4..].as_mut_ptr();
                let w1 = &ow[(i + n) * board::CELL_2D .. (i + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum4: x86_64::__m128;
                unsafe {
                    sum4 = x86_64::_mm_setzero_ps();
                }
                const M : usize = 16;
                let bit4 = 0x0f;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let bi1 = bit4 & (black >> idx) as usize;
                    let wi1 = bit4 & (white >> idx) as usize;
                    let bi2 = bit4 & (black >> (idx + 4)) as usize;
                    let wi2 = bit4 & (white >> (idx + 4)) as usize;
                    let bi3 = bit4 & (black >> (idx + 8)) as usize;
                    let wi3 = bit4 & (white >> (idx + 8)) as usize;
                    let bi4 = bit4 & (black >> (idx + 12)) as usize;
                    let wi4 = bit4 & (white >> (idx + 12)) as usize;

                    unsafe {
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

                        let x41 = x86_64::_mm_load_ps(w1.as_ptr().add(idx));
                        let x42 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 4));
                        let x43 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 8));
                        let x44 = x86_64::_mm_load_ps(w1.as_ptr().add(idx + 12));

                        let m1 = x86_64::_mm_mul_ps(c1, x41);
                        let m2 = x86_64::_mm_mul_ps(c2, x42);
                        let m3 = x86_64::_mm_mul_ps(c3, x43);
                        let m4 = x86_64::_mm_mul_ps(c4, x44);
                        let sum12 = x86_64::_mm_add_ps(m1, m2);
                        let sum34 = x86_64::_mm_add_ps(m3, m4);
                        let sum1234 = x86_64::_mm_add_ps(sum12, sum34);
                        sum4 = x86_64::_mm_add_ps(sum4, sum1234);
                    }
                }
                unsafe {
                    x86_64::_mm_store_ps(res4, sum4);
                }
            }

            unsafe {
                let mut x1 = x86_64::_mm_load_ps(sum44[0..].as_ptr());
                let mut x2 = x86_64::_mm_load_ps(sum44[4..].as_ptr());
                let mut x3 = x86_64::_mm_load_ps(sum44[8..].as_ptr());
                let mut x4 = x86_64::_mm_load_ps(sum44[12..].as_ptr());
                let mut x5 = x86_64::_mm_load_ps(sum44[16..].as_ptr());
                let mut x6 = x86_64::_mm_load_ps(sum44[20..].as_ptr());
                let mut x7 = x86_64::_mm_load_ps(sum44[24..].as_ptr());
                let mut x8 = x86_64::_mm_load_ps(sum44[28..].as_ptr());

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

                x86_64::_mm_store_ps(hid.as_mut_ptr().add(i), y4);
                x86_64::_mm_store_ps(hid.as_mut_ptr().add(i + 4), y42);
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias();
        let wh = self.wlayer1();
        let wdc1 = self.wl1bias();
        let wh2 = self.wlayer2();
        let mut hid2 = [0f32 ; N_HIDDEN2];
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = wdc1[i];
            let mut sum4 = [0f32 ; 4];
            let mut s4 = unsafe {x86_64::_mm_setzero_ps()};
            for j in (0..N_HIDDEN).step_by(16) {
                let idx = i * N_HIDDEN + j;
                unsafe {
                    let x1 = x86_64::_mm_load_ps(hid.as_ptr().add(j));
                    let x2 = x86_64::_mm_load_ps(hid.as_ptr().add(j + 4));
                    let x3 = x86_64::_mm_load_ps(hid.as_ptr().add(j + 8));
                    let x4 = x86_64::_mm_load_ps(hid.as_ptr().add(j + 12));
                    let w1 = x86_64::_mm_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 4));
                    let w3 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 8));
                    let w4 = x86_64::_mm_load_ps(wh.as_ptr().add(idx + 12));
                    let mul1 = x86_64::_mm_mul_ps(x1, w1);
                    let mul2 = x86_64::_mm_mul_ps(x2, w2);
                    let mul3 = x86_64::_mm_mul_ps(x3, w3);
                    let mul4 = x86_64::_mm_mul_ps(x4, w4);
                    let s12 = x86_64::_mm_add_ps(mul1, mul2);
                    let s34 = x86_64::_mm_add_ps(mul3, mul4);
                    // let s12 = x86_64::_mm_fmadd_ps(x3, w3, mul1);
                    // let s34 = x86_64::_mm_fmadd_ps(x4, w4, mul2);
                    let s1234 = x86_64::_mm_add_ps(s12, s34);
                    s4 = x86_64::_mm_add_ps(s1234, s4);
                }
            }
            unsafe {x86_64::_mm_store_ps(sum4.as_mut_ptr(), s4);}
            for h in sum4 {
                hidsum2 += h;
            }
            // res += wh2[i] / ((-hidsum2).exp() + 1f32);
            hid2[i] = hidsum2;
        }
        unsafe {  // relu
            let h1 = x86_64::_mm_load_ps(hid2.as_ptr());
            let h2 = x86_64::_mm_load_ps(hid2.as_ptr().add(4));
            let h3 = x86_64::_mm_load_ps(hid2.as_ptr().add(8));
            let h4 = x86_64::_mm_load_ps(hid2.as_ptr().add(12));
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
            x86_64::_mm_store_ps(hid2.as_mut_ptr(), y1234);
        }
        for h in hid2.iter().take(4) {
            res += h;
        }
        res
    }

    #[cfg(target_arch="aarch64")]
    pub fn evaluatev7bb_simd_mul(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        
        let (fsb, fsw) = ban.fixedstones();
        
        let ow = self.wban();
        let wtbn = self.wteban();
        let wfs = self.wfixedstones();
        let wdc = self.wibias();
        const N : usize = 16;
        let mut hid = [0f32 ; N_HIDDEN];
        for i in (0..N_HIDDEN).step_by(N) {
            let mut sumn = [0.0f32 ; N];

            for n in 0..N {
                let w1 = &ow[(i + n) * bitboard::CELL_2D .. ];
                for y in (0..board::NUMCELL).step_by(2) {
                    let bit8 = 0xff;
                    let idx = y * bitboard::NUMCELL;
                    let bi1 = bit8 & (black >> idx) as usize;
                    let wi1 = bit8 & (white >> idx) as usize;
                    let bi3 = bit8 & (black >> (idx + bitboard::NUMCELL)) as usize;
                    let wi3 = bit8 & (white >> (idx + bitboard::NUMCELL)) as usize;
        
                    unsafe {
                        let b12 = vld1q_f32_x2(TBL8_BIT2F32.addr(bi1));
                        let w12 = vld1q_f32_x2(TBL8_BIT2F32.addr(wi1));
                        let b34 = vld1q_f32_x2(TBL8_BIT2F32.addr(bi3));
                        let w34 = vld1q_f32_x2(TBL8_BIT2F32.addr(wi3));

                        let c1 = vsubq_f32(b12.0, w12.0);
                        let c2 = vsubq_f32(b12.1, w12.1);
                        let c3 = vsubq_f32(b34.0, w34.0);
                        let c4 = vsubq_f32(b34.1, w34.1);
                        let w = vld1q_f32_x4(w1.as_ptr().add(idx));
                        let w1 = vmulq_f32(w.0, c1);
                        let w12 = vmulq_f32(w.1, c2);
                        let w2 = vmulq_f32(w.2, c3);
                        let w22 = vmulq_f32(w.3, c4);
                        let sum = vaddq_f32(w1, w12);
                        let sum2 = vaddq_f32(w2, w22);
                        let sum = vaddvq_f32(vaddq_f32(sum, sum2));
                        sumn[n] += sum;
                    }
                }
            }
            unsafe {
                let sum4 = vld1q_f32_x4(sumn.as_ptr());

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
                // vst1q_f32_x4(sumn.as_mut_ptr(), float32x4x4_t(sum41, sum42, sum43, sum44));
                vst1q_f32(sumn.as_mut_ptr(), sum41);
                vst1q_f32(sumn.as_mut_ptr().add(4), sum42);
                vst1q_f32(sumn.as_mut_ptr().add(8), sum43);
                vst1q_f32(sumn.as_mut_ptr().add(12), sum44);
            }
            for n in 0 .. N {
                // sumn[n] = (-sumn[n]).exp();
                hid[i + n] = 1f32 / ((-sumn[n]).exp() + 1.0);
                // res += wh[i + n] / ((-sumn[n]).exp() + 1.0);
            }  // 2050nps
        }
        // 2nd layer to output
        let mut res = self.wl2bias();
        let wh = self.wlayer1();
        let wdc1 = self.wl1bias();
        let wh2 = self.wlayer2();
        let mut hid2 = [0f32 ; N_HIDDEN2];
        for i in 0..N_HIDDEN2 {
            let mut hidsum = wdc1[i];
            for j in (0..N_HIDDEN).step_by(16) {
               unsafe {
                    let inp = vld1q_f32_x4(hid.as_ptr().add(j));
                    let wei = vld1q_f32_x4(wh.as_ptr().add(i * N_HIDDEN + j));
                    let mul0 = vmulq_f32(inp.0, wei.0);
                    let mul1 = vmulq_f32(inp.1, wei.1);
                    let mul2 = vmlaq_f32(mul0, inp.2, wei.2);
                    let mul3 = vmlaq_f32(mul1, inp.3, wei.3);
                    let add4 = vaddq_f32(mul2, mul3);
                    hidsum += vaddvq_f32(add4);
                }
            }
            // res += wh2[i] / ((-hidsum).exp() + 1f32);
            // hid2[i] = (-hidsum).exp() + 1f32;
            hid2[i] = (-hidsum).exp();
        }
        unsafe {
            let inp = vld1q_f32_x4(hid2.as_ptr());
            let wei = vld1q_f32_x4(wh2.as_ptr());
            // let mul0 = vdivq_f32(wei.0, inp.0);
            // let mul1 = vdivq_f32(wei.1, inp.1);
            // let mul2 = vdivq_f32(wei.2, inp.2);
            // let mul3 = vdivq_f32(wei.3, inp.3);
            let one = vdupq_n_f32(1f32);
            let inp0 = vaddq_f32(one, inp.0);
            let inp1 = vaddq_f32(one, inp.1);
            let inp2 = vaddq_f32(one, inp.2);
            let inp3 = vaddq_f32(one, inp.3);
            let mul0 = vdivq_f32(wei.0, inp0);
            let mul1 = vdivq_f32(wei.1, inp1);
            let mul2 = vdivq_f32(wei.2, inp2);
            let mul3 = vdivq_f32(wei.3, inp3);
            let mul12 = vaddq_f32(mul0, mul1);
            let mul34 = vaddq_f32(mul2, mul3);
            let add4 = vaddq_f32(mul12, mul34);
            res += vaddvq_f32(add4);
        }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev3bb_simdavx(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];

        const N : usize = 8;
        let mut sumarr : [f32 ; N] = [0.0 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum88 : [f32 ; N * 8] = [0.0 ; N * 8];

            for n in 0..N {
                let res8 = sum88[n * 8..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum8: x86_64::__m256;
                unsafe {
                    sum8 = x86_64::_mm256_setzero_ps();
                }
                const M : usize = 32;
                let mut bit8 : u64 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let b81 = (bit8 & black) >> (4 * j);
                    let w81 = (bit8 & white) >> (4 * j);
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> (4 * j + 1);
                    let w82 = (bit8 & white) >> (4 * j + 1);
                    bit8 <<= 1;
                    let b83 = (bit8 & black) >> (4 * j + 2);
                    let w83 = (bit8 & white) >> (4 * j + 2);
                    bit8 <<= 1;
                    let b84 = (bit8 & black) >> (4 * j + 3);
                    let w84 = (bit8 & white) >> (4 * j + 3);
                    bit8 <<= 1;

                    unsafe {
                        let c1 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b81 as i64),
                                x86_64::_mm_set1_epi64x(w81 as i64));
                        let c2 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b82 as i64),
                                x86_64::_mm_set1_epi64x(w82 as i64));
                        let c3 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b83 as i64),
                                x86_64::_mm_set1_epi64x(w83 as i64));
                        let c4 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b84 as i64),
                                x86_64::_mm_set1_epi64x(w84 as i64));

                        let c81 = x86_64::_mm256_cvtepi8_epi32(c1);
                        let c82 = x86_64::_mm256_cvtepi8_epi32(c2);
                        let c83 = x86_64::_mm256_cvtepi8_epi32(c3);
                        let c84 = x86_64::_mm256_cvtepi8_epi32(c4);

                        let f81 = x86_64::_mm256_cvtepi32_ps(c81);
                        let f82 = x86_64::_mm256_cvtepi32_ps(c82);
                        let f83 = x86_64::_mm256_cvtepi32_ps(c83);
                        let f84 = x86_64::_mm256_cvtepi32_ps(c84);

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
                    }
                }
                unsafe {
                    x86_64::_mm256_storeu_ps(res8, sum8);
                }
            }

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

                // teban
                let wtbn = x86_64::_mm256_load_ps(wtbn.as_ptr().add(hidx));
                let tbn = x86_64::_mm256_set1_ps(teban);
                let tbn4 = x86_64::_mm256_mul_ps(wtbn, tbn);
                let h18 = x86_64::_mm256_add_ps(h18, tbn4);
                // fixed stones
                let wfsb4 = x86_64::_mm256_load_ps(wfs.as_ptr().add(hidx));
                let fsb = x86_64::_mm256_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm256_mul_ps(wfsb4, fsb);
                let wfsw4 = x86_64::_mm256_load_ps(
                        wfs.as_ptr().add(hidx + N_HIDDEN));
                let fsw = x86_64::_mm256_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm256_mul_ps(wfsw4, fsw);
                let fsbw = x86_64::_mm256_add_ps(fsb4, fsw4);
                let h18 = x86_64::_mm256_add_ps(h18, fsbw);
                // dc
                let wdc4 = x86_64::_mm256_load_ps(wdc.as_ptr().add(hidx));
                let h1234 = x86_64::_mm256_add_ps(h18, wdc4);

                let emx4 = Weight::expmx_ps_simd256(h1234);
                let one = x86_64::_mm256_set1_ps(1.0);
                let hsp14 = x86_64::_mm256_add_ps(emx4, one);
                let wh4 = x86_64::_mm256_load_ps(wh.as_ptr().add(hidx));

                let y4 = x86_64::_mm256_div_ps(wh4, hsp14);
                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                x86_64::_mm256_store_ps(sumarr.as_mut_ptr(), y4);
            }
            for s in sumarr {
                sum += s;
            }
            // sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        sum
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev7bb_simdavx(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;

        let fs = ban.fixedstones();

        let ow = self.wban();
        let wtbn = self.wteban();
        let wfs = self.wfixedstones();
        let wdc = self.wibias();
        const N : usize = 16;
        let mut hid = [0f32 ; N_HIDDEN];
        let mut sumn = [0f32 ; N];

        for i in (0..N_HIDDEN).step_by(N) {
            let hidx = i;

            for m in (0..N).step_by(8) {
                let mut sum88 = [0f32 ; N * 8];
                for n in 0..N / 2 {
                    let idx = hidx + m + n;
                    let res8 = sum88[n * 8..].as_mut_ptr();
                    let w1 = &ow[(idx) * board::CELL_2D .. (idx + 1) * board::CELL_2D];
                    // let mut hidsum : f32 = dc[i];
                    let mut sum8 = unsafe {x86_64::_mm256_setzero_ps()};
                    const M : usize = 32;
                    let bit8 = 0xff;
                    for j in 0..board::CELL_2D / M {
                        let idx = j * M;
                        let bi1 = bit8 & (black >> idx) as usize;
                        let wi1 = bit8 & (white >> idx) as usize;
                        let bi2 = bit8 & (black >> (idx + 8)) as usize;
                        let wi2 = bit8 & (white >> (idx + 8)) as usize;
                        let bi3 = bit8 & (black >> (idx + 16)) as usize;
                        let wi3 = bit8 & (white >> (idx + 16)) as usize;
                        let bi4 = bit8 & (black >> (idx + 24)) as usize;
                        let wi4 = bit8 & (white >> (idx + 24)) as usize;

                        unsafe {
                            let b81 = x86_64::_mm256_load_ps(
                                TBL8_BIT2F32.addr(bi1));
                            let b82 = x86_64::_mm256_load_ps(
                                TBL8_BIT2F32.addr(bi2));
                            let b83 = x86_64::_mm256_load_ps(
                                TBL8_BIT2F32.addr(bi3));
                            let b84 = x86_64::_mm256_load_ps(
                                TBL8_BIT2F32.addr(bi4));
                            let w81 = x86_64::_mm256_load_ps(
                                TBL8_BIT2F32.addr(wi1));
                            let w82 = x86_64::_mm256_load_ps(
                                TBL8_BIT2F32.addr(wi2));
                            let w83 = x86_64::_mm256_load_ps(
                                TBL8_BIT2F32.addr(wi3));
                            let w84 = x86_64::_mm256_load_ps(
                                TBL8_BIT2F32.addr(wi4));
                            let f81 = x86_64::_mm256_sub_ps(b81, w81);
                            let f82 = x86_64::_mm256_sub_ps(b82, w82);
                            let f83 = x86_64::_mm256_sub_ps(b83, w83);
                            let f84 = x86_64::_mm256_sub_ps(b84, w84);

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
                        }
                    }
                    unsafe {
                        x86_64::_mm256_store_ps(res8, sum8);
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
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i), y41);
                x86_64::_mm256_storeu_ps(hid.as_mut_ptr().add(i + 8), y42);
            }
        }

        // 2nd layer to output
        let mut res = self.wl2bias();
        let wh = self.wlayer1();
        let wdc1 = self.wl1bias();
        let wh2 = self.wlayer2();

        let mut hid2 = [0f32 ; N_HIDDEN2];
        for i in 0..N_HIDDEN2 {
            let mut hidsum2 = wdc1[i];
            let mut sum4 = [0f32 ; N_HIDDEN / 4];
            for j in (0..N_HIDDEN).step_by(16) {
                let idx = i * N_HIDDEN + j;
                unsafe {
                    let x1 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j));
                    let x2 = x86_64::_mm256_loadu_ps(hid.as_ptr().add(j + 8));
                    let w1 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx));
                    let w2 = x86_64::_mm256_load_ps(wh.as_ptr().add(idx + 8));
                    let mul1 = x86_64::_mm256_mul_ps(x1, w1);
                    let mul2 = x86_64::_mm256_mul_ps(x2, w2);
                    let s12 = x86_64::_mm256_add_ps(mul1, mul2);
                    let s1 = x86_64::_mm256_extractf128_ps(s12, 0);
                    let s2 = x86_64::_mm256_extractf128_ps(s12, 1);
                    let s4 = x86_64::_mm_add_ps(s1, s2);
                    x86_64::_mm_store_ps(sum4.as_mut_ptr().add(j / 4), s4);
                }
            }
            for h in sum4 {
                hidsum2 += h;
            }
            hid2[i] = hidsum2;
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
            let s1 = x86_64::_mm256_extractf128_ps(y3, 0);
            let s2 = x86_64::_mm256_extractf128_ps(y3, 1);
            let s4 = x86_64::_mm_add_ps(s1, s2);
            x86_64::_mm_store_ps(hid2.as_mut_ptr(), s4);
        }
        for h in hid2.iter().take(4) {
            res += h;
        }
        res
    }

    #[cfg(target_arch="x86_64")]
    pub fn evaluatev3bb_simdavx2(&self, ban : &bitboard::BitBoard) -> f32 {
        let black = ban.black;
        let white = ban.white;
        let teban = ban.teban as f32;
        let ow = &self.weight;

        let fs = ban.fixedstones();

        let mut sum = *ow.last().unwrap();

        let wtbn = &ow[board::CELL_2D * N_HIDDEN .. (board::CELL_2D + 1)* N_HIDDEN];
        let wfs = &ow[(board::CELL_2D + 1) * N_HIDDEN .. (board::CELL_2D + 1 + 2) * N_HIDDEN];
        let wdc = &ow[(board::CELL_2D + 1 + 2) * N_HIDDEN .. (board::CELL_2D + 1 + 2 + 1) * N_HIDDEN];
        let wh = &ow[(board::CELL_2D + 1 + 2 + 1) * N_HIDDEN ..];

        const N : usize = 4;
        let mut sumarr : [f32 ; N] = [0.0 ; N];

        for i in 0..N_HIDDEN / N {
            let hidx = i * N;
            let mut sum48 : [f32 ; N * 8] = [0.0 ; N * 8];

            for n in 0..N {
                let res8 = sum48[n * 8..].as_mut_ptr();
                let w1 = &ow[(hidx + n) * board::CELL_2D .. (hidx + n + 1) * board::CELL_2D];
                // let mut hidsum : f32 = dc[i];
                let mut sum8: x86_64::__m256;
                unsafe {
                    sum8 = x86_64::_mm256_setzero_ps();
                }
                const M : usize = 32;
                let mut bit8 : u64 = 0x0101010101010101;
                for j in 0..board::CELL_2D / M {
                    let idx = j * M;
                    let b81 = (bit8 & black) >> (4 * j);
                    let w81 = (bit8 & white) >> (4 * j);
                    bit8 <<= 1;
                    let b82 = (bit8 & black) >> (4 * j + 1);
                    let w82 = (bit8 & white) >> (4 * j + 1);
                    bit8 <<= 1;
                    let b83 = (bit8 & black) >> (4 * j + 2);
                    let w83 = (bit8 & white) >> (4 * j + 2);
                    bit8 <<= 1;
                    let b84 = (bit8 & black) >> (4 * j + 3);
                    let w84 = (bit8 & white) >> (4 * j + 3);
                    bit8 <<= 1;

                    unsafe {
                        let c1 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b81 as i64),
                                x86_64::_mm_set1_epi64x(w81 as i64));
                        let c2 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b82 as i64),
                                x86_64::_mm_set1_epi64x(w82 as i64));
                        let c3 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b83 as i64),
                                x86_64::_mm_set1_epi64x(w83 as i64));
                        let c4 = x86_64::_mm_sub_epi8(
                                x86_64::_mm_set1_epi64x(b84 as i64),
                                x86_64::_mm_set1_epi64x(w84 as i64));

                        let c81 = x86_64::_mm256_cvtepi8_epi32(c1);
                        let c82 = x86_64::_mm256_cvtepi8_epi32(c2);
                        let c83 = x86_64::_mm256_cvtepi8_epi32(c3);
                        let c84 = x86_64::_mm256_cvtepi8_epi32(c4);

                        let f81 = x86_64::_mm256_cvtepi32_ps(c81);
                        let f82 = x86_64::_mm256_cvtepi32_ps(c82);
                        let f83 = x86_64::_mm256_cvtepi32_ps(c83);
                        let f84 = x86_64::_mm256_cvtepi32_ps(c84);

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
                    }
                }
                unsafe {
                    x86_64::_mm256_storeu_ps(res8, sum8);
                }
            }

            unsafe {
                let x11 = x86_64::_mm_load_ps(sum48.as_ptr());
                let x12 = x86_64::_mm_load_ps(sum48.as_ptr().add(4));
                let x21 = x86_64::_mm_load_ps(sum48.as_ptr().add(8));
                let x22 = x86_64::_mm_load_ps(sum48.as_ptr().add(12));
                let mut x1 = x86_64::_mm_add_ps(x11, x12);
                let mut x2 = x86_64::_mm_add_ps(x21, x22);

                let x31 = x86_64::_mm_load_ps(sum48.as_ptr().add(16));
                let x32 = x86_64::_mm_load_ps(sum48.as_ptr().add(20));
                let x41 = x86_64::_mm_load_ps(sum48.as_ptr().add(24));
                let x42 = x86_64::_mm_load_ps(sum48.as_ptr().add(28));
                let mut x3 = x86_64::_mm_add_ps(x31, x32);
                let mut x4 = x86_64::_mm_add_ps(x41, x42);

                x86_64::_MM_TRANSPOSE4_PS(&mut x1, &mut x2, &mut x3, &mut x4);

                let h12 = x86_64::_mm_add_ps(x1, x2);
                let h34 = x86_64::_mm_add_ps(x3, x4);
                let h1234 = x86_64::_mm_add_ps(h12, h34);

                // teban
                let wtbn = x86_64::_mm_load_ps(wtbn.as_ptr().add(hidx));
                let tbn = x86_64::_mm_set1_ps(teban);
                let tbn4 = x86_64::_mm_mul_ps(wtbn, tbn);
                let h1234 = x86_64::_mm_add_ps(h1234, tbn4);
                // fixed stones
                let wfsb4 = x86_64::_mm_load_ps(wfs.as_ptr().add(hidx));
                let fsb = x86_64::_mm_set1_ps(fs.0 as f32);
                let fsb4 = x86_64::_mm_mul_ps(wfsb4, fsb);
                let wfsw4 = x86_64::_mm_load_ps(
                        wfs.as_ptr().add(hidx + N_HIDDEN));
                let fsw = x86_64::_mm_set1_ps(fs.1 as f32);
                let fsw4 = x86_64::_mm_mul_ps(wfsw4, fsw);
                let fsbw = x86_64::_mm_add_ps(fsb4, fsw4);
                let h1234 = x86_64::_mm_add_ps(h1234, fsbw);
                // dc
                let wdc4 = x86_64::_mm_load_ps(wdc.as_ptr().add(hidx));
                let h1234 = x86_64::_mm_add_ps(h1234, wdc4);

                let emx4 = Weight::expmx_ps_simd(h1234);
                let one = x86_64::_mm_set1_ps(1.0);
                let hsp14 = x86_64::_mm_add_ps(emx4, one);
                let wh4 = x86_64::_mm_load_ps(wh.as_ptr().add(hidx));

                let y4 = x86_64::_mm_div_ps(wh4, hsp14);
                // let rhsp14 = x86_64::_mm_rcp_ps(hsp14);
                // let two = x86_64::_mm_set1_ps(2.0);
                // let x2 = x86_64::_mm_mul_ps(rhsp14, hsp14);
                // let x3 = x86_64::_mm_sub_ps(two, x2);
                // let x4 = x86_64::_mm_mul_ps(rhsp14, x3);
                // let y4 = x86_64::_mm_mul_ps(w24, x4);

                x86_64::_mm_store_ps(sumarr.as_mut_ptr(), y4);
            }
            // for s in sumarr {
            //     sum += s;
            // }
            sum += sumarr[0] + sumarr[1] + sumarr[2] + sumarr[3];
        }
        sum
    }

    #[allow(dead_code)]
    fn learn(&mut self, _ban : &board::Board, _winner : i8, _eta : f32) {
    }

    #[allow(dead_code)]
    fn learnbb(&mut self, _ban : &bitboard::BitBoard, _winner : i8, _eta : f32) {
    }

    #[allow(dead_code)]
    fn learnbbdiff(&self, _ban : &bitboard::BitBoard, _winner : i8, _eta : f32, _dfw : &mut Weight) {
    }

    #[allow(dead_code)]
    fn learnbbminib(&self, banscores : &[&(bitboard::BitBoard, i8)], eta : f32, dfw : &mut Weight) {
        for (ban , winner) in banscores.iter() {
            self.learnbbdiff(ban, *winner, eta, dfw);
        }
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
        let ban = board::Board::from(rfen).unwrap();
        ban.put();
        let mut w = weight::Weight::new();
        w.init();
        let res_nosimde = w.evaluatev7bb(&bban);
        let res_simd = w.evaluatev7bb_simd(&bban);
        let res_simdavx = w.evaluatev7bb_simdavx(&bban);
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
        let ban = board::Board::from(rfen).unwrap();
        ban.put();
        let mut w = weight::Weight::new();
        w.init();
        let res_nosimdy = w.evaluatev7(&ban);
        let res_nosimdi = w.evaluatev7bb(&bban);
        let res_simdmul = w.evaluatev7bb_simd_mul(&bban);
        // let res_simd = w.evaluatev7bb_simd(&bban);
        assert!(dbg_assert_eq(&res_nosimdy, &res_nosimdi));
        assert!(dbg_assert_eq(&res_nosimdi, &res_simdmul));
        // println!("{res_nosimd} == {res_simd} == {res_simdavx} ???");
    }
}
