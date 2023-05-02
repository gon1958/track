use crate::constants::*;
use crate::registry::*;
use crate::track_node::*;

use chrono::Local;
use mnist::*;
use ndarray::prelude::*;
use rand::Rng;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::{fs, thread};

pub const CURRENT_MUTATION_FILE: &str = "mutation";

pub fn run_mutate2(fname: Option<String>, iter_count: u32) {
    let mut reg = new_regs();
    let mut track_proc = NodeTr::new(MAX_TREE_NODES, reg.gen_root_node());
    match fname {
        Some(f) => {
            let contents = fs::read_to_string(f).expect("Initial trac file read error.");
            track_proc.set_root(string_to_tree(&contents));
        }
        None => {
            track_proc.build_tree(&mut reg);
        }
    };

    let mut global_min_diff_val: f32 = 0.0;
    for idx in 0..RW_LEN {
        if idx == 0 {
            global_min_diff_val += (1. - reg.read_reg_value_sigmoid(idx as f32)).powi(2);
        } else {
            global_min_diff_val += (0. - reg.read_reg_value_sigmoid(idx as f32)).powi(2);
        }
    }

    let mut iter_num: u32 = 0;
    let mut over_error:u32=0;
    fs::write(&CURRENT_VERSION_FILE, track_proc.tree_to_string())
        .expect("Current version file write error");
    loop {
        iter_num += 1;
        if iter_num > iter_count {
            break;
        }
        let mut min_diff_idx: i32 = -1;
        let mut min_diff_val = global_min_diff_val;

        let length_train_set =
            rand::thread_rng().gen_range(TRAINING_SAMPLE_LENGTH_MIN..=TRAINING_SAMPLE_LENGTH_MAX);
        let begin_train_set =
            rand::thread_rng().gen_range(0..TRAINING_SET_LENGTH - length_train_set);
        let mut mut_num = 0;
        loop {
            let mut curr_diff_idx = 0;
            if min_diff_idx != 0 {
                curr_diff_idx = 0;
            } else {
                curr_diff_idx = 1;
            }

            track_proc.mutate_tree(&mut reg);
            let count_alternative_mut_max =
                (track_proc.get_root().node_count() as f32 * RATE_ALTERNATIVE_MUT) as usize;
            let fname = CURRENT_MUTATION_FILE.to_owned() + &curr_diff_idx.to_string();
            fs::write(fname, track_proc.tree_to_string()).expect("Current mutate file write error");

            let curr_diff_val = check_tree(&begin_train_set, &length_train_set, curr_diff_idx)
                / length_train_set as f32;
            if curr_diff_val < min_diff_val || min_diff_idx == -1 {
                min_diff_idx = curr_diff_idx as i32;
                min_diff_val = curr_diff_val;
            }
            mut_num += 1;
            if min_diff_val < global_min_diff_val || mut_num > count_alternative_mut_max {
                break;
            };
            let tree = string_to_tree(
                &fs::read_to_string(CURRENT_VERSION_FILE).expect("Current mutate file read error"),
            );
            track_proc.set_root(tree);
        }
        if min_diff_val < global_min_diff_val {
            global_min_diff_val = min_diff_val;
        }else{
            over_error += 1;
        }
        let fname = CURRENT_MUTATION_FILE.to_owned() + &min_diff_idx.to_string();
        let tree =
            string_to_tree(&fs::read_to_string(fname).expect("Initial version file read error"));
        track_proc.set_root(tree);
        let fname = CURRENT_MUTATION_FILE.to_owned() + &min_diff_idx.to_string();
        fs::rename(&fname, &CURRENT_VERSION_FILE).expect("Rename file error");
        println!(
            "time={} iter={} global_min_diff_val={} over_error={} node_count={} mut_num={}",
            Local::now().format("%H:%M:%S").to_string(),
            iter_num,
            global_min_diff_val,
            over_error,
            track_proc.get_root().node_count(),
            mut_num,
        );
    }
}

fn check_tree(start_train_idx: &usize, length_train_set: &usize, step_num: usize) -> f32 {
    let Mnist {
        trn_img, trn_lbl, ..
    } = MnistBuilder::new()
        .label_format_digit()
        .training_set_length(TRAINING_SET_LENGTH as u32)
        .validation_set_length(10_000)
        .test_set_length(10_000)
        .finalize();

    // Can use an Array2 or Array3 here (Array3 for visualization)
    let train_data = Array3::from_shape_vec((TRAINING_SET_LENGTH, 28, 28), trn_img.to_vec())
        .expect("Error converting images to Array3 struct")
        .map(|x| *x as f32 / 256.0);

    // Convert the returned Mnist struct to Array2 format
    let train_labels: Array2<f32> =
        Array2::from_shape_vec((TRAINING_SET_LENGTH, 1), trn_lbl.to_vec())
            .expect("Error converting training labels to Array2 struct")
            .map(|x| *x as f32);

    let mut sum_diff: f32 = 0.;
    let (tmc, rmc) = mpsc::channel();
    let mut closed_threads = 0 as usize;

    let mut handles = Vec::new();

    for image_num in *start_train_idx..start_train_idx + length_train_set {
        let tds = train_data.slice(s![image_num, .., ..]);
        let tls = train_labels.slice(s![image_num, ..]);
        let ansv = tls[0 as usize] as usize;
        let mut reg = new_regs();

        for idx in 0..reg.get_rw_size() {
            reg.write_rwreg_value(0 as f32, idx as f32);
        }

        let mut idx = 0;
        for e in tds.iter() {
            reg.write_roreg_value(*e, idx);
            idx += 1;
        }

        while handles.len() - closed_threads > MAX_ACTIVE_THREAD {
            let received = rmc.recv().unwrap();
            closed_threads += 1;
            if received >= 0. {
                sum_diff += received;
            }
        }

        let tmc1 = tmc.clone();
        handles.push(run_thread(reg, ansv, tmc1, step_num));
    }

    let final_handle = thread::spawn(move || {
        tmc.send(-1.).unwrap();
    });

    for received in rmc {
        if received >= 0. {
            sum_diff += received;
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
    final_handle.join().unwrap();
    sum_diff
}

fn run_thread(mut reg: Regs, ansv: usize, tmc: Sender<f32>, step_num: usize) -> JoinHandle<()> {
    let handle = thread::spawn(move || {
        let fname = CURRENT_MUTATION_FILE.to_owned() + &step_num.to_string();
        let tree =
            string_to_tree(&fs::read_to_string(fname).expect("Current mutate file read error"));
        node_calc(tree.root(), &mut reg);

        let mut pow2_diff = 0. as f32;
        for idx in 0..reg.get_rw_size() {
            if idx == ansv {
                pow2_diff += (1. - reg.read_reg_value_sigmoid(idx as f32)).powi(2);
            } else {
                pow2_diff += (0. - reg.read_reg_value_sigmoid(idx as f32)).powi(2);
            }
        }
        tmc.send(pow2_diff).unwrap();
    });
    handle
}
