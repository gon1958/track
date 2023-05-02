use crate::registry::*;
use crate::track_func::*;
use crate::track_node::*;
use crate::weighted_random::*;
use mnist::*;
use ndarray::prelude::*;
use std::fs;
use trees::*;

pub fn trac_func(){

}

pub fn weight_rnd() {
    // let &mut lookup:Vec<u32> = &mut Vec::new();
    let mut weights: Vec<u32> = vec![1, 1, 3, 3];
    let total_weight: u32 = 0;
    let mut lookup: Vec<u32> = Vec::new();
    let total_weight = calc_lookups(&weights, &mut lookup);
    //....
    // Каждый раз когда вам необходимо выбрать случайный элемент:
    println!(
        "{}",
        weighted_random( &mut weights, &mut lookup, total_weight)
    );
}

pub fn mnist() {
    // Deconstruct the returned Mnist struct.
    let Mnist {
        trn_img,
        trn_lbl,
        tst_img,
        tst_lbl,
        ..
    } = MnistBuilder::new()
        .label_format_digit()
        .training_set_length(50_000)
        .validation_set_length(10_000)
        .test_set_length(10_000)
        .finalize();

    let image_num = 0;
    // Can use an Array2 or Array3 here (Array3 for visualization)
    let train_data = Array3::from_shape_vec((50_000, 28, 28), trn_img)
        .expect("Error converting images to Array3 struct")
        .map(|x| *x as f32 / 256.0);
    println!("{:#.1?}\n", train_data.slice(s![image_num, .., ..]));

    // Convert the returned Mnist struct to Array2 format
    let train_labels: Array2<f32> = Array2::from_shape_vec((50_000, 1), trn_lbl)
        .expect("Error converting training labels to Array2 struct")
        .map(|x| *x as f32);
    println!(
        "The first digit is a {:?}",
        train_labels.slice(s![image_num, ..])
    );

    let _test_data = Array3::from_shape_vec((10_000, 28, 28), tst_img)
        .expect("Error converting images to Array3 struct")
        .map(|x| *x as f32 / 256.);

    let _test_labels: Array2<f32> = Array2::from_shape_vec((10_000, 1), tst_lbl)
        .expect("Error converting testing labels to Array2 struct")
        .map(|x| *x as f32);
}

pub fn mutate() {
    let mut r = simple_init_registry();
    let t = r.gen_rnd_trac_node();
    let mut n = NodeTr::new(100, t);
    println!(
        "m0 {} {}",
        n.tree_to_string(),
        n.get_root().root().node_count()
    );
    n.mutate_tree(&mut r);
    println!("m0 {}", n.tree_to_string());
    n.build_tree(&mut r);
    for i in 0..200{
        n.mutate_tree(&mut r);
        println!("nodes={}", n.get_root().root().node_count());
    }
}

pub fn calc() {
    let mut r = simple_init_registry();
    let mut n = NodeTr::new(5_000, r.gen_root_node());
    let tree: Tree<f32> = tr(ADD as f32)
        / (tr(SUB as f32)
            / (tr(DIV as f32) / tr(0 as f32) / tr(1 as f32))
            / (tr(ADD as f32) / tr(2 as f32) / tr(3 as f32)))
        / (tr(MULT as f32)
            / (tr(SUB as f32) / tr(4 as f32) / tr(5 as f32))
            / (tr(MULT as f32) / tr(6 as f32) / tr(7 as f32)));
    n.set_root(tree);
    assert_eq!(3., n.calc_tree(&mut r));
}

pub fn to_string() {
    let mut r = simple_init_registry();
    let mut n = NodeTr::new(5_000, r.gen_root_node());

    // let mut tree: Tree<f32> = tr(ADD as f32)
    //     / tr(1 as f32)
    //     / (tr(DIV as f32) / tr(3 as f32) / tr(5 as f32))
    //     / tr(4 as f32)
    //     / (tr(MULT as f32) / tr(8 as f32) / tr(7 as f32))
    //     / tr(9 as f32);
    let tree: Tree<f32> = tr(ADD as f32)
        / (tr(SUB as f32)
            / (tr(DIV as f32) / tr(0 as f32) / tr(1 as f32))
            / (tr(ADD as f32) / tr(2 as f32) / tr(3 as f32)))
        / (tr(MULT as f32)
            / (tr(SUB as f32) / tr(4 as f32) / tr(5 as f32))
            / (tr(MULT as f32) / tr(6 as f32) / tr(7 as f32)));

    // let mut tree=Tree::<i32>::from_tuple(   (2,(1,7,(4,(3,5,),9,),4,4,(8,7,),9,),)  );
    // let mut tree = trees::Tree::<f32>::from_tuple((2.,(1.,5.,(3.,5.,),4.,4.,(8.,7.,),9.,)));
    // let mut tree = trees::Tree::<i32>::from_tuple(  (5,(7,(4,(5,(6,78,),3,(46,42,), ), ),36,), ) );
    // let mut tree = tr( 5)/(tr(7)/(tr(4)/(tr(5)/(tr(6)/tr(78))/tr(3)/(tr(46)/tr(42))))/tr(36));

    n.set_root(tree);
    println!("q0 {}", n.tree_to_string());
    n.mutate_tree(&mut r);
    println!(
        "q1 {} {}",
        n.tree_to_string(),
        n.get_root().node_count()
    );
    let s1 = n.tree_to_string();
    let t = string_to_tree(&s1);
    let s2 = tree_to_string(t.root());
    assert_eq!(s1, s2);
    println!("q2 {}", tree_to_string(t.root()));
}

pub fn to_file() {
    let mut r = simple_init_registry();
    let mut n = NodeTr::new(10_000, r.gen_root_node());

    for idx in 1..=100000 {
        n.mutate_tree(&mut r);
    }
    // println!("w1 {} {}", tree_to_string(tree.root()), tree.root().node_count());
    println!("cnt ={}", n.get_root().node_count() ,
    );
    fs::write("gon.txt", n.tree_to_string()).expect("Write error");

    simple_init_registry();
    println!("res={:.5}", n.calc_tree(&mut r));
    println!("upper={} lower={} ", r.get_higest_rw_value(), r.get_lower_rw_value());

    let contents = fs::read_to_string("gon.txt").expect("Read error");
    let tree = string_to_tree(&contents);
    simple_init_registry();
    println!("res={:.5}", n.calc_tree(&mut r));
    println!("upper={} lower={}", r.get_higest_rw_value(), r.get_lower_rw_value());
}
