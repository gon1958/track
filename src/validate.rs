use crate::registry::*;
use crate::track_func::*;
use crate::track_node::*;
use chrono::Local;
use mnist::*;
use ndarray::prelude::*;
use std::cmp;
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::{fs, thread};

use crate::constants::*;

pub fn run_validate() {
    let mut total_current_matches: Vec<u32> = vec![0; RW_LEN];
    for start_train_idx in (0..TEST_SET_LENGTH).step_by(PARALLEL_RUN_SET_SIZE) {
        let current_matches = check_tree(&start_train_idx);
        for idx in 0..RW_LEN {
            total_current_matches[idx] += current_matches[idx];
        }
    }
    let current_total_matches : u32 = total_current_matches.iter().sum();
    for idx in 0..RW_LEN {
        println!(
            "val={} cnt={}",
            idx,
            total_current_matches[idx],
        );
    }
}

fn check_tree(start_test_idx: &usize) -> Vec<u32> {
    let Mnist {
        trn_img,
        trn_lbl,
        tst_img,
        tst_lbl,
        ..
    } = MnistBuilder::new()
        .label_format_digit()
        .training_set_length(TRAINING_SET_LENGTH as u32)
        .validation_set_length(10_000)
        .test_set_length(10_000)
        .finalize();

    let test_data = Array3::from_shape_vec((TEST_SET_LENGTH, 28, 28), tst_img)
        .expect("Error converting images to Array3 struct")
        .map(|x| *x as f32 / 256.);

    let test_labels: Array2<f32> = Array2::from_shape_vec((TEST_SET_LENGTH, 1), tst_lbl)
        .expect("Error converting testing labels to Array2 struct")
        .map(|x| *x as f32);

    let mut match_count = vec![0 as u32; RW_LEN];
    let (tmc, rmc) = mpsc::channel();
    let mut closed_threads = 0 as usize;

    let mut handles = Vec::new();

    let final_test_idx: usize =
        cmp::min(*start_test_idx + PARALLEL_RUN_SET_SIZE, TEST_SET_LENGTH);
    for image_num in *start_test_idx..final_test_idx {
        let tds = test_data.slice(s![image_num, .., ..]);
        let tls = test_labels.slice(s![image_num, ..]);
        let ansv = tls[0 as usize] as i32;
        let mut re = new_regs();

        for idx in 0..re.get_rw_size() {
            re.write_rwreg_value(0 as f32, idx as f32);
        }

        let mut idx = 0;
        for e in tds.iter() {
            re.write_roreg_value(*e, idx);
            idx += 1;
        }

        while handles.len() - closed_threads > MAX_ACTIVE_THREAD {
            let received = rmc.recv().unwrap();
            closed_threads += 1;
            if received != -1 {
                match_count[received as usize] += 1;
            }
        }

        let tmc1 = tmc.clone();
        handles.push(run_thread(re, ansv, tmc1));
    }

    let final_handle = thread::spawn(move || {
        tmc.send(-1).unwrap();
    });

    for received in rmc {
        if received != -1 {
            match_count[received as usize] += 1;
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
    final_handle.join().unwrap();

    match_count
}

fn run_thread(mut re: Regs, ansv: i32, tmc: Sender<i32>) -> JoinHandle<()> {
    let handle = thread::spawn(move || {
        let tree = string_to_tree(
            &fs::read_to_string(CURRENT_VERSION_FILE).expect("Current mutate file read error"),
        );
        node_calc(tree.root(), &mut re);

        let mut max_val = 0. as f32;
        let mut max_idx = -1 as i32;
        for idx in 0..re.get_rw_size() {
            if re.read_reg_value(idx as f32) > max_val {
                max_val = re.read_reg_value(idx as f32);
                max_idx = idx as i32;
            }
        }
        if max_idx == ansv {
            tmc.send(ansv).unwrap();
        } else {
            tmc.send(-1).unwrap();
        };
    });
    handle
}
