use crate::registry::*;
use rand::Rng;
use trees::{Node, Tree};

use crate::track_func::*;
use std::fmt::Display;
use std::pin::Pin;

pub struct NodeTr {
    root: Tree<f32>,
    max_nodes: usize,
}

impl NodeTr {
    pub fn new(n: usize, r:Tree<f32>) -> NodeTr {
        NodeTr {
            max_nodes: n,
            root: r,
        }
    }

    pub fn set_root(&mut self, r: Tree<f32>) {
        self.root = r;
    }

    pub fn get_root(&self) -> &Tree<f32> {
        &self.root
    }

    pub fn build_tree(&mut self, re:&mut Regs) {
        while self.root.root().node_count() <= self.max_nodes {
            self.mutate_tree(re);
        }
    }

    pub fn mutate_tree(&mut self, re:&mut Regs) {
        let mut node_num = rand::thread_rng().gen_range(1..=self.root.root().node_count() - 1);
        let can_truncate = self.root.root().node_count() > self.max_nodes;
        let r = self.root.root_mut();
        mutate_node(
            r,
            &mut node_num,
            re,
            can_truncate,
        );
    }

    pub fn calc_tree(&self, re:&mut Regs) -> f32 {
        node_calc(self.root.root(), re)
    }

    pub fn tree_to_string(&self) -> String {
        tree_to_string(self.root.root())
    }
}

pub fn node_calc(node: &Node<f32>, re: &mut Regs) -> f32 {
    if node.has_no_child() {
        re.read_reg_value(*node.data())
    } else {
        let arg: Vec<f32> = node.iter().map(|x| node_calc(x, re)).collect();
        re.func_call(*node.data(), &arg)
    }
}

fn mutate_node(
    mut node: Pin<&mut Node<f32>>,
    node_num: &mut usize,
    re: &mut Regs,
    can_truncate: bool,
) {
    let mut node_idx = 0usize;
    let mut iter = node.iter_mut();
    loop {
        let n = iter.next();
        match n {
            Some(t) => {
                // println!("ee d={} cnt={} num={} idx={}", t.data(),  t.node_count(), *node_num, node_idx);
                if *node_num == 1 as usize {
                    // println!("qq root={} cnt={} idx={}", node.data(),  node.node_count(), node_idx);
                    break;
                } else if t.node_count() < *node_num as usize {
                    *node_num -= t.node_count();
                    node_idx += 1 as usize;
                } else if t.node_count() >= *node_num {
                    *node_num -= 1;
                    mutate_node(t, node_num, re, can_truncate);
                    return;
                }
            }
            None => {
                return;
            }
        }
    }
    if node.iter_mut().nth(node_idx).unwrap().has_no_child() {
        mutate_reg(node, node_idx, re);
    } else {
        mutate_func(node, node_idx, re, can_truncate);
    }
}

fn mutate_reg(mut node: Pin<&mut Node<f32>>, node_idx: usize, re: &mut Regs) {
    let mut child = Tree::new(0.0);

    if rand::thread_rng().gen_range(0..=1) == 0 as usize {
        //mutate to reg number
        if *node.data() == WRITE as f32 && node_idx == 1 as usize {
            child = Tree::new(re.rnd_write_idx() as f32);
        } else {
            child = Tree::new(re.rnd_read_idx() as f32);
        }
    } else {
        // mutate to func
        child = re.gen_rnd_trac_node();
    }
    node.iter_mut()
        .nth(node_idx)
        .unwrap()
        .insert_next_sib(child);
    node.iter_mut().nth(node_idx).unwrap().detach();
}

fn mutate_func(
    mut node: Pin<&mut Node<f32>>,
    node_idx: usize,
    re: &mut Regs,
    can_truncate: bool,
) {
    let mut new_node: Tree<f32> = Tree::new(re.get_funcs().gen_rnd_func_all() as f32);
    let mut old_node = node.iter_mut().nth(node_idx).unwrap();

    if arg_count(*new_node.data() as u8) < arg_count(*old_node.data() as u8) {
        new_node = mutate_func_trunc(new_node, old_node, re, can_truncate);
    } else {
        (new_node, old_node) = move_args(new_node, old_node, re);
    }

    node.iter_mut()
        .nth(node_idx)
        .unwrap()
        .insert_next_sib(new_node);
    node.iter_mut().nth(node_idx).unwrap().detach();
}

fn mutate_func_trunc<'a>(
    mut new_node: Tree<f32>,
    mut old_node: Pin<&'a mut Node<f32>>,
    re: &mut Regs,
    can_truncate: bool,
) -> Tree<f32> {
    let new_args = arg_count(*new_node.data() as u8);
    let old_args = arg_count(*old_node.data() as u8);

    if old_args == 4 as usize && new_args == 1 as usize {
        let mut new_node1: Tree<f32> = Tree::new(re.get_funcs().gen_rnd_func2() as f32);
        let mut new_node2: Tree<f32> = Tree::new(re.get_funcs().gen_rnd_func2() as f32);

        if can_truncate {
            old_node = delete_rnd_arg(old_node);
        }

        (new_node1, old_node) = move_args(new_node1, old_node, re);
        (new_node2, old_node) = move_args(new_node2, old_node, re);

        let mut new_node3: Tree<f32> = Tree::new(re.get_funcs().gen_rnd_func2() as f32);
        new_node3.push_back(new_node1);
        new_node3.push_back(new_node2);

        new_node.push_back(new_node3);
    } else if old_args == 4 as usize && new_args == 2 as usize {
        let mut new_node1: Tree<f32> = Tree::new(re.get_funcs().gen_rnd_func2() as f32);
        let mut new_node2: Tree<f32> = Tree::new(re.get_funcs().gen_rnd_func2() as f32);

        if can_truncate {
            old_node = delete_rnd_arg(old_node);
        }

        (new_node1, old_node) = move_args(new_node1, old_node, re);
        (new_node2, old_node) = move_args(new_node2, old_node, re);

        new_node.push_back(new_node1);
        new_node.push_back(new_node2);
    } else if old_args == 2 as usize && new_args == 1 as usize {
        if can_truncate {
            old_node = delete_rnd_arg(old_node);
        }
        (new_node, old_node) = move_args(new_node, old_node, re);
    }
    new_node
}

fn delete_rnd_arg<'a>(mut old_node: Pin<&'a mut Node<f32>>) -> Pin<&'a mut Node<f32>> {
    let old_args = old_node.degree();
    old_node
        .iter_mut()
        .nth(rand::thread_rng().gen_range(0..old_args))
        .unwrap()
        .detach();
    old_node
}

fn move_args<'a>(
    mut new_node: Tree<f32>,
    mut old_node: Pin<&'a mut Node<f32>>,
    r: &mut Regs,
) -> (Tree<f32>, Pin<&'a mut Node<f32>>) {
    for idx in 0..arg_count(*new_node.data() as u8) {
        if old_node.has_no_child() {
            new_node.push_back(Tree::new(r.rnd_read_idx() as f32));
        } else {
            let old_args = old_node.degree();
            new_node.push_back(
                old_node
                    .iter_mut()
                    .nth(rand::thread_rng().gen_range(0..old_args))
                    .unwrap()
                    .detach(),
            );
        }
    }
    (new_node, old_node)
}

pub fn tree_to_string<T: Display>(node: &Node<T>) -> String {
    if node.has_no_child() {
        node.data().to_string()
    } else {
        format!(
            "{}({})",
            node.data(),
            node.iter()
                .fold(String::new(), |s, c| format!("{}{},", s, tree_to_string(c)))
        )
    }
}

fn find_item(s: &str) -> (char, usize) {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b'(' {
            return ('(', i);
        } else if item == b',' {
            return (',', i);
        } else if item == b')' {
            return (')', i);
        }
    }
    (' ', s.len())
}

fn scan_string<'a>(s: &'a str, node: &'a mut Tree<f32>) -> usize {
    let mut pos = 0 as usize;
    loop {
        let (delim, idx) = find_item(&s[pos..]);
        let val: f32 = match (&s[pos..pos + idx].to_string()).trim().parse::<f32>() {
            Ok(num) => num,
            Err(_) => 0.0,
        };
        pos += idx + 1;
        if delim == '(' {
            let mut n = Tree::new(val);
            pos += scan_string(&s[pos..], &mut n);
            node.push_back(n);
        } else if delim == ',' {
            if idx > 0 {
                node.push_back(Tree::new(val));
            }
        } else if delim == ')' {
            return pos;
        }
        if pos == s.len() - 1 {
            return pos;
        }
    }
}

pub fn string_to_tree(tree_str: &String) -> Tree<f32> {
    let (delim, len) = find_item(&tree_str[..]);
    if delim != '(' {
        panic!("Wrong string fromat");
    }

    let val: f32 = match (&tree_str[..len].to_string()).trim().parse::<f32>() {
        Ok(num) => num,
        Err(_) => panic!("Wrong node fromat"),
    };

    let mut tree = Tree::new(val);
    scan_string(&tree_str[len + 1..], &mut tree);
    tree
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::simple_init_registry;
    use trees::tr;
    use trees::Tree;

    #[test]
    fn test01() {
        let mut r = simple_init_registry();
        let mut n = NodeTr::new(5_000, r.gen_rnd_trac_node());
        let mut t: Tree<f32> = Tree::new(MULT as f32);
        t.push_back(Tree::new(1 as f32));
        t.push_back(Tree::new(7 as f32));
        assert_eq!("4(1,7,)", tree_to_string(t.root()));

        let tree: Tree<f32> =
            tr(ADD as f32) / tr(1 as f32) / (tr(DIV as f32) / tr(3 as f32) / tr(5 as f32));
        assert_eq!("2(1,5(3,5,),)", tree_to_string(tree.root()));

        let tree: trees::Tree<f32> = trees::Tree::<f32>::from_tuple((
            SUB as f32,
            1 as f32,
            2 as f32,
            (DIV as f32, 4 as f32, 5 as f32),
        ));
        assert_eq!("3(1,2,5(4,5,),)", tree_to_string(tree.root()));
        assert_eq!(6, tree.node_count());

        let tree: Tree<f32> =
            tr(ADD as f32) / tr(5 as f32) / (tr(DIV as f32) / tr(6 as f32) / tr(7 as f32));
        assert_eq!("2(5,5(6,7,),)", tree_to_string(tree.root()));
        n.set_root(tree);
        assert_eq!(0.5, n.calc_tree(&mut r));
    }

    #[test]
    fn test02() {
        let mut r = simple_init_registry();
        let mut n = NodeTr::new(5_000, r.gen_rnd_trac_node());
        let tree: Tree<f32> = tr(ADD as f32)
            / (tr(SUB as f32)
                / (tr(DIV as f32) / tr(0 as f32) / tr(1 as f32))
                / (tr(ADD as f32) / tr(2 as f32) / tr(3 as f32)))
            / (tr(MULT as f32)
                / (tr(SUB as f32) / tr(4 as f32) / tr(5 as f32))
                / (tr(MULT as f32) / tr(6 as f32) / tr(7 as f32)));

        let s1 = tree_to_string(tree.root());
        let t = string_to_tree(&s1);
        let s2 = tree_to_string(t.root());
        assert_eq!(s1, s2);
        n.set_root(tree);
        assert_eq!(3., n.calc_tree(&mut r));
    }

    #[test]
    fn test03() {
        let mut r = simple_init_registry();
        let mut n = NodeTr::new(5_000, r.gen_rnd_trac_node());
        for i in 1..=100 {
            n.mutate_tree(&mut r);
            // println!("{} {}", n.tree_to_string(), n.get_root().node_count());
            n.calc_tree(&mut r);
        }
        // 6(0(6(5,2,),2(3(2(2,6,),1(4,2,2,4,),),0,),  1(5,3(7,1,),3,8,)                 ,5(0,8,),),5(6,1,),)
        // 6(0(6(5,2,),2(3(2(2,6,),1(4,2,2,4,),),0,),  7(0( 6(8,3,), 6(3(7,1,),5,) ,),)  ,5(0,8,),),5(6,1,),)

        //6(  1(9,0(5,5(1,6,),1,6,),6,4,),                  4(3(4,2(7,2,),),5,),)
        //6(  7(1(5(6,9,), 1(4, 0(5,5(1,6,),1,6,) ,2,8,) ,),),  4(3(4,2(7,2,),),5,),)
    }
}
