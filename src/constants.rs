use crate::registry::*;
use crate::track_func::*;

pub const CURRENT_VERSION_FILE: &str = "current_version.track";
pub const ROLLBACK_FILE: &str = "rollback.track";
pub const TRAINING_SET_LENGTH: usize = 50_000;
pub const TRAINING_SAMPLE_LENGTH_MIN: usize = 5_000;
pub const TRAINING_SAMPLE_LENGTH_MAX: usize = 10_000;
pub const TEST_SET_LENGTH: usize = 10_000;
pub const MAX_TREE_NODES: usize = 2;
pub const PARALLEL_RUN_SET_SIZE: usize = 10_000;
pub const MAX_ACTIVE_THREAD: usize = 3;
pub const RATE_ALTERNATIVE_MUT:f32=1.;
pub const RW_LEN : usize = 10;
pub const RO_LEN : usize = 28 * 28;

pub fn new_regs() -> Regs {
    let ft = FuncTr::new();
    // ft.set_weights(vec![1, 1, 1, 1, 1, 1, 7, 1]);

    let mut r = Regs::new(RW_LEN, RO_LEN,  vec![0.; RO_LEN], ft);
    // r.set_rw_max_value(1.);
    // r.set_positive_rw_value(true);
    // let mut  w = Vec::new();
    //
    // for idx in 0..rw_len{
    //     w.push((ro_len/2) as u32);
    // }
    // for idx in 0..ro_len{
    //     w.push(1 as u32);
    // }
    // r.set_weights(w);
    r
}
