extern crate backtrace_on_stack_overflow;
extern crate chrono;
extern crate core;

mod debug;
mod registry;
mod track_func;
mod track_node;
mod weighted_random;
mod mutate1;
mod validate;
mod constants;
mod mutate2;

use crate::track_node::*;
use crate::mutate1::*;
use crate::mutate2::*;
use crate::validate::*;
use std::env;
use std::thread;

fn main() {
    // print full error stack
    // unsafe{backtrace_on_stack_overflow::enable()}

    // Get start track program from arg
    let args: Vec<String> = env::args().collect();
    let mut fname = None;
    let mut iter_count: u32 = 1_000_000;

    if args.len() >= 2{
        iter_count = args[1].parse().unwrap();
    }

    if args.len() >= 3 {
        fname =Some(args[2].clone());
    }

    // run mutate
    let builder = thread::Builder::new()
        .name("reductor".into())
        .stack_size(32 * 1024 * 1024);
    let handler = builder
        .spawn(move|| {
            run_mutate2(fname, iter_count);
        })
        .unwrap();
    handler.join().unwrap();

    // check result
    let builder = thread::Builder::new()
        .name("reductor".into())
        .stack_size(32 * 1024 * 1024);
    let handler = builder
        .spawn(|| {
            run_validate();
        })
        .unwrap();
    handler.join().unwrap();

    // misc debug modules
    // debug::trac_func();
    // debug::mutate();
    // debug::calc();
    // debug::to_string();
    // debug::to_file();
    // debug::mnist();
    // debug::weight_rnd();
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::BorrowMut;
    use trees::walk::Visit;
    use trees::{bfs, fr, tr, RcNode, Size, Tree, TreeWalk};

    #[test]
    fn test01() {
        let mut tree = Tree::new(9);
        let root = tree.root();
        assert!(root.has_no_child());
        assert_eq!(root.data(), &9);

        let mut root = tree.root_mut();
        *root.data_mut() = 0;
        assert_eq!(root.data(), &0);

        tree.push_back(Tree::new(1));
        tree.push_back(Tree::new(2));
        assert_eq!(tree.to_string(), "0( 1 2 )");

        let mut iter = tree.iter();
        assert_eq!(iter.next().unwrap().data(), &1);
        assert_eq!(iter.next().unwrap().data(), &2);

        assert_eq!(tree.front().unwrap().data(), &1);
        assert_eq!(tree.back().unwrap().data(), &2);
        {
            let mut node_1 = tree.front_mut().unwrap();
            node_1.push_back(Tree::new(3));
            node_1.push_back(Tree::new(4));
            node_1.push_back(Tree::new(5));
            let tree_4 = node_1.iter_mut().nth(1).unwrap().detach();
        }
        assert_eq!(tree.to_string(), "0( 1( 3 5 ) 2 )");
        assert_eq!(2, tree.root().degree());
        assert_eq!(5, tree.root().node_count());

        {
            let mut node_1 = tree.front_mut().unwrap();
            let tree_3 = node_1.iter_mut().nth(0).unwrap();
            let mut node_3 = tree.front_mut().unwrap();
            node_3.push_back(Tree::new(4));
            assert_eq!(tree.to_string(), "0( 1( 3 5 4 ) 2 )");
        }
        {
            let mut node_1 = tree.front_mut().unwrap();
            let tree_3 = node_1.iter_mut().nth(0).unwrap();
            let mut node_3 = tree.front_mut().unwrap();
            node_3.append(trees::Forest::new());
            assert_eq!(tree.to_string(), "0( 1( 3 5 4 ) 2 )");
        }
    }

    #[test]
    fn test02() {
        let tree = trees::Tree::<i32>::from_tuple((0, (1, 2, 3), (4, 5, 6)));
        assert_eq!(tree.to_string(), "0( 1( 2 3 ) 4( 5 6 ) )");

        let forest = trees::Forest::<i32>::from_tuple(((1, 2, 3), (4, 5, 6)));
        assert_eq!(forest.to_string(), "( 1( 2 3 ) 4( 5 6 ) )");
    }

    #[test]
    fn test03() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6));
        let str_repr = "0( 1( 2 3 ) 4( 5 6 ) )";
        assert_eq!(tree.to_string(), str_repr);

        assert_eq!(fr::<i32>().to_string(), "()");

        let forest = -(tr(1) / tr(2) / tr(3)) - (tr(4) / tr(5) / tr(6));
        let str_repr = "( 1( 2 3 ) 4( 5 6 ) )";
        assert_eq!(forest.to_string(), str_repr);
    }

    #[test]
    fn test04() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / (tr(5) / tr(7) / tr(8)) / tr(6));
        assert_eq!(tree.to_string(), "0( 1( 2 3 ) 4( 5( 7 8 ) 6 ) )");
    }

    #[test]
    fn test07() {
        let tree = Tree::<i32>::from_tuple((0, (1, 2, 3), (4, 5, 6)));
        let root = RcNode::from(tree);
        let tree = unsafe { root.into_tree() };
    }

    #[test]
    fn test08() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6));
        assert_eq!(&0, tree.root().data());
        let mut iter = tree.iter();
        assert_eq!(2 as usize, iter.len());
        assert_eq!(&1, iter.next().unwrap().data());
        assert_eq!(1 as usize, iter.len());
        assert_eq!(&4, iter.next().unwrap().data());
        assert_eq!(0 as usize, iter.len());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test09() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6));
        assert_eq!(&0, tree.root().data());
        let node = tree.root();
        let mut iter = node.iter();
        let node = iter.next();
        assert_eq!(&1, node.unwrap().data());
        {
            let mut iter1 = node.unwrap().iter();
            let node1 = iter1.next();
            assert_eq!(&2, node1.unwrap().data());
            let node1 = iter1.next();
            assert_eq!(&3, node1.unwrap().data());
        }
        let node = iter.next();
        assert_eq!(&4, node.unwrap().data());
    }

    #[test]
    fn test10() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6));
        let mut walk = TreeWalk::from(tree);

        match walk.get() {
            Some(Visit::Begin(b)) => assert_eq!(&0, b.data()),
            Some(Visit::End(e)) => assert_eq!(&0, e.data()),
            Some(Visit::Leaf(l)) => assert_eq!(&0, l.data()),
            _ => println!("other"),
        }
        let s = walk.borrow_mut();
        walk.forward();
    }

    #[test]
    fn test11() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6));
        let v = tree.into_bfs();
        let mut i = v.iter;
        assert_eq!(0, i.next().unwrap().data);
        assert_eq!(1, i.next().unwrap().data);
        assert_eq!(4, i.next().unwrap().data);
        assert_eq!(2, i.next().unwrap().data);
        assert_eq!(3, i.next().unwrap().data);
        assert_eq!(5, i.next().unwrap().data);
        assert_eq!(6, i.next().unwrap().data);
        assert_eq!(None, i.next());
    }

    #[test]
    fn next() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6));
        let mut walk = TreeWalk::from(tree);
        assert_eq!(
            walk.next(),
            Some(Visit::Begin((tr(1) / tr(2) / tr(3)).root()))
        );
        assert_eq!(walk.next(), Some(Visit::Leaf(tr(2).root())));
        assert_eq!(walk.next(), Some(Visit::Leaf(tr(3).root())));
        assert_eq!(
            walk.next(),
            Some(Visit::End((tr(1) / tr(2) / tr(3)).root()))
        );
        assert_eq!(
            walk.next(),
            Some(Visit::Begin((tr(4) / tr(5) / tr(6)).root()))
        );
        assert_eq!(walk.next(), Some(Visit::Leaf((tr(5)).root())));
        assert_eq!(walk.next(), Some(Visit::Leaf((tr(6)).root())));
        assert_eq!(
            walk.next(),
            Some(Visit::End((tr(4) / tr(5) / tr(6)).root()))
        );
        assert_eq!(
            walk.next(),
            Some(Visit::End(
                (tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6))).root()
            ))
        );
    }

    #[test]
    fn to_child() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6));
        let mut walk = TreeWalk::from(tree);
        assert_eq!(
            walk.get(),
            Some(Visit::Begin(
                (tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6))).root()
            ))
        );
        walk.to_child(1);
        assert_eq!(
            walk.get(),
            Some(Visit::Begin((tr(4) / tr(5) / tr(6)).root()))
        );
        walk.to_parent();
        walk.to_child(0);
        assert_eq!(
            walk.get(),
            Some(Visit::Begin((tr(1) / tr(2) / tr(3)).root()))
        );
    }

    #[test]
    fn test12() {
        let tree = tr(0) / (tr(1) / tr(2) / tr(3)) / (tr(4) / tr(5) / tr(6));
        let walk = TreeWalk::from(tree);
        let o = walk.get();
        if let Some(v) = o {
            let n = v.node();
            assert_eq!(0, *n.data());
        }
    }
}
