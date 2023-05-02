use crate::registry::*;
use crate::track_node::*;
use crate::constants::*;

use chrono::Local;
use mnist::*;
use ndarray::prelude::*;
use std::cmp;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::{fs, thread};

pub fn run_mutate1(fname: Option<String>, iter_count: u32) {
    let mut reg = new_regs();
    let mut nod_tr = NodeTr::new(MAX_TREE_NODES, reg.gen_root_node());
    match fname {
        Some(f) => {
            let contents = fs::read_to_string(f).expect("Initial trac file read error.");
            nod_tr.set_root(string_to_tree(&contents));
        }
        None => {
            nod_tr.build_tree(&mut reg);
        }
    };

    let mut best_matches: u32 = 0;
    let mut iter_num: u32 = 0;
    let mut reset_num: u32 = 0;
    loop {
        fs::write(&ROLLBACK_FILE, nod_tr.tree_to_string()).expect("Rollback file write error");
        nod_tr.mutate_tree(&mut reg);
        iter_num += 1;
        fs::write(CURRENT_VERSION_FILE, nod_tr.tree_to_string()).expect("Current mutate file write error");
        let mut total_current_matches: Vec<u32> = vec![0; RW_LEN];
        for start_train_idx in (0..TRAINING_SET_LENGTH).step_by(PARALLEL_RUN_SET_SIZE) {
            let current_matches = check_tree(&start_train_idx);
            for idx in 0..RW_LEN{
                total_current_matches[idx] += current_matches[idx];
            }
        }
        let current_total_matches = total_current_matches.iter().sum();
        if current_total_matches < best_matches {
            let contents = fs::read_to_string(ROLLBACK_FILE).expect("Rollback file read error");
            nod_tr.set_root(string_to_tree(&contents));
            reset_num += 1;
        }
        if current_total_matches > best_matches {
            for idx in 0..RW_LEN{
                println!("idx={} count={}", idx, total_current_matches[idx]);
            }
            best_matches = current_total_matches;
            fs::write(
                format!("{}-{}-{}.track", MAX_TREE_NODES, best_matches, iter_num),
                nod_tr.tree_to_string(),
            )
                .expect("Next trac program version Write error");
            println!(
                "time={} iter={} reset={} nodes={} matches={}",
                Local::now().format("%H:%M:%S").to_string(),
                iter_num,
                reset_num,
                nod_tr.get_root().node_count(),
                current_total_matches,
            );
        }
        if iter_num >= iter_count || current_total_matches >= TRAINING_SET_LENGTH as u32{
            break;
        }
    }
}

fn check_tree(start_train_idx: &usize) -> Vec<u32> {
    let Mnist {
        trn_img,
        trn_lbl,
        ..
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
    let train_labels: Array2<f32> = Array2::from_shape_vec((TRAINING_SET_LENGTH, 1), trn_lbl.to_vec())
        .expect("Error converting training labels to Array2 struct")
        .map(|x| *x as f32);

    let mut match_count = vec![0 as u32; RW_LEN];
    let (tmc, rmc) = mpsc::channel();
    let mut closed_threads = 0 as usize;

    let mut handles = Vec::new();

    let final_train_idx: usize = cmp::min(*start_train_idx + PARALLEL_RUN_SET_SIZE, TRAINING_SET_LENGTH);
    for image_num in *start_train_idx..final_train_idx {
        let tds = train_data.slice(s![image_num, .., ..]);
        let tls = train_labels.slice(s![image_num, ..]);
        let ansv = tls[0 as usize] as i32;
        let mut re = new_regs();

        // let mut  w = Vec::new();
        for idx in 0..re.get_rw_size() {
            re.write_rwreg_value(0 as f32, idx as f32);
            // w.push(1 as u32);
        }

        let mut idx = 0;
        for e in tds.iter() {
            re.write_roreg_value(*e, idx);
            idx += 1;
            // w.push(((*e) * 256.) as u32);
        }
        // re.set_weights(w);

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
        let tree = string_to_tree(&fs::read_to_string(CURRENT_VERSION_FILE).expect("Current mutate file read error"));
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
