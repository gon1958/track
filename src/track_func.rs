use rand::Rng;

pub const EQ: u8 = 0;
pub const GT: u8 = 1;
pub const ADD: u8 = 2;
pub const SUB: u8 = 3;
pub const MULT: u8 = 4;
pub const DIV: u8 = 5;
pub const WRITE: u8 = 6;
pub const READ: u8 = 7;

pub struct FuncTr {
    funcs: Vec<u8>,
    weights: Vec<u32>,
}

impl FuncTr {
    pub fn new() -> FuncTr {
        FuncTr {
            funcs: vec![EQ, GT, ADD, SUB, MULT, DIV, WRITE, READ],
            weights: vec![1; 8],
        }
    }

    pub fn set_weights(&mut self, w: Vec<u32>) {
        if w.len() != self.funcs.len() {
            panic!("Incorrect weights length.");
        }
        self.weights.clear();
        self.weights = w.to_vec();
    }

    fn gen_rnd_func(&self, funs: &[u8], wgt: &[u32]) -> u8 {
        let total: u32 = wgt.iter().sum();
        let num: u32 = rand::thread_rng().gen_range(1..=total);
        let mut n: u32 = 0;
        for idx in 0..wgt.len() {
            n += wgt[idx];
            if n >= num {
                return funs[idx];
            }
        }
        funs[funs.len()]
    }

    pub fn gen_rnd_func_all(&self) -> u8 {
        self.gen_rnd_func(&self.funcs[..], &self.weights[..])
    }

    pub fn gen_rnd_func2(&self) -> u8 {
        self.gen_rnd_func(&self.funcs[2..=6], &self.weights[2..=6])
    }

    pub fn gen_rnd_func42(self) -> u8 {
        self.gen_rnd_func(&self.funcs[..=6], &self.weights[..=6])
    }
}

pub fn func_to_string(f_idx: u8) -> String {
    if f_idx == EQ {
        "EQ".to_string()
    } else if f_idx == GT {
        "GT".to_string()
    } else if f_idx == ADD {
        "ADD".to_string()
    } else if f_idx == SUB {
        "SUB".to_string()
    } else if f_idx == MULT {
        "MULT".to_string()
    } else if f_idx == DIV {
        "DIV".to_string()
    } else if f_idx == WRITE {
        "WRITE".to_string()
    } else {
        "READ".to_string()
    }
}


pub fn arg_count(f_idx: u8) -> usize {
    if f_idx == EQ {
        // if &1 = &2 then &3 else &4
        4
    } else if f_idx == GT {
        // if &1 > &2 then &3 else &4
        4
    } else if f_idx == ADD {
        // &1 + &2
        2
    } else if f_idx == SUB {
        // &1 - &2
        2
    } else if f_idx == MULT {
        // &1 * &2
        2
    } else if f_idx == DIV {
        // &1 / &2
        2
    } else if f_idx == WRITE {
        // read &1 and write to &2
        // WRINTE &1.value  to &2.registry
        2
    } else {
        // READ &1.value
        1
    }
}

#[cfg(test)]
mod tests {
    use crate::track_func::*;
    use crate::simple_init_registry;

    #[test]
    fn test01() {
        let f = EQ;
        assert_eq!("EQ", func_to_string(f));
        assert_eq!(4, arg_count(f));
    }

    #[test]
    fn test02(){
        let r= simple_init_registry();
        let tf = FuncTr::new();
        let f2= tf.gen_rnd_func2();
        assert!(f2 >= 2 && f2 <= 6);
        println!("{}", tf.gen_rnd_func2());
    }
}
