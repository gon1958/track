use crate::track_func::*;
use crate::weighted_random::*;
use rand::Rng;
use std::f64::consts::E;
use trees::{tr, Tree};

pub struct Regs {
    rw_size: usize,
    ro_size: usize,
    rw_max_value: f32,
    higest_rw_value: f32,
    lower_rw_value: f32,
    positive_rw_value: bool,
    registry: Vec<f32>,
    use_weights : bool,
    weights: Vec<u32>,
    funcs: FuncTr,
    total_weight: u32,
    lookup: Vec<u32>,
}

impl Regs {
    pub fn new(w_len: usize, r_len: usize, ro_val: Vec<f32>, f: FuncTr) -> Regs {
        let mut r = Regs {
            rw_size: w_len,
            ro_size: r_len,
            rw_max_value: 0.,
            higest_rw_value: 0.,
            lower_rw_value: 0.,
            registry: vec![0.; w_len + r_len],
            use_weights: false,
            weights: vec![1; w_len + r_len],
            positive_rw_value: false,
            funcs: f,
            total_weight: 0,
            lookup: Vec::new(),
        };
        if ro_val.len() > r_len {
            panic!("Init registry vector too big.");
        }
        for idx in 0..ro_val.len() - 1 {
            r.registry[w_len + idx] = ro_val[idx];
        }
        r.init_weight();
        r
    }

    fn init_weight(&mut self) {
        self.lookup = Vec::new();
        self.total_weight = calc_lookups(&self.weights, &mut self.lookup);
    }

    pub fn set_weights(&mut self, w: Vec<u32>) {
        if w.len() != self.registry.len() {
            panic!("Incorrect weights length.");
        }
        self.weights.clear();
        self.weights = w.to_vec();
        self.init_weight();
    }

    pub fn set_positive_rw_value(&mut self, s: bool) {
        self.positive_rw_value = s;
    }

    pub fn set_rw_max_value(&mut self, m: f32) {
        self.rw_max_value = m;
    }

    pub fn get_funcs(&self) -> &FuncTr {
        &self.funcs
    }

    pub fn get_rw_size(&self) -> usize {
        self.rw_size
    }

    pub fn get_higest_rw_value(&self) -> f32 {
        self.higest_rw_value
    }

    pub fn get_lower_rw_value(&self) -> f32 {
        self.lower_rw_value
    }

    pub fn rnd_read_idx(&mut self) -> usize {
        if self.use_weights {
            weighted_random(&self.weights, &mut self.lookup, self.total_weight)
        }else{
            rand::thread_rng().gen_range(0..=self.rw_size + self.ro_size - 1)
        }
    }

    pub fn rnd_write_idx(&self) -> usize {
        rand::thread_rng().gen_range(0..=self.rw_size - 1)
    }

    pub fn read_reg_value(&mut self, val: f32) -> f32 {
        let idx = (val % (self.rw_size + self.ro_size - 1) as f32) as usize;
        self.registry[idx]
    }

    pub fn read_reg_value_sigmoid(&mut self, val: f32) -> f32 {
        let idx = (val % (self.rw_size + self.ro_size - 1) as f32) as usize;
        if idx >= self.rw_size {
            self.registry[idx]
        } else {
            1. / (1. + (E as f32).powf(-self.registry[idx]  ))
        }
    }

    pub fn write_roreg_value(&mut self, value: f32, idx: usize) {
        self.registry[self.rw_size + idx] = value;
    }

    pub fn write_rwreg_value(&mut self, value: f32, idx: f32) -> f32 {
        let real_idx: usize = (idx % (self.rw_size) as f32) as usize;
        if value > self.higest_rw_value {
            self.higest_rw_value = value;
        }
        if value < self.lower_rw_value {
            self.lower_rw_value = value;
        }
        let mut real_value = value;
        if self.rw_max_value > 0. {
            real_value = real_value % self.rw_max_value;
        }
        if self.positive_rw_value && real_value < 0. {
            real_value = 0.;
        }
        self.registry[real_idx] = real_value;
        real_value
    }

    pub fn func_call(&mut self, f_idx: f32, args: &Vec<f32>) -> f32 {
        if arg_count(f_idx as u8) != args.len() {
            panic!(
                "Incorrect argument count {} for function {}.",
                args.len(),
                f_idx
            );
        }

        let mut retval: f32 = 0.0;
        let idx = f_idx as u8;
        if idx == EQ {
            // println!("EQ {} {} {} {}", args[0], args[1], args[2], args[3]);
            if args[0] == args[1] {
                retval = args[2]
            } else {
                retval = args[3]
            }
        } else if idx == GT {
            // println!("EQ {} {} {} {}", args[0], args[1], args[2], args[3]);
            if args[0] > args[1] {
                retval = args[2]
            } else {
                retval = args[3]
            }
        } else if idx == ADD {
            // println!("ADD {} {}", args[0], args[1]);
            retval = args[0] + args[1]
        } else if idx == SUB {
            // println!("SUB {} {}", args[0], args[1]);
            retval = args[0] - args[1]
        } else if idx == MULT {
            // println!("MULT {} {}", args[0], args[1]);
            retval = args[0] * args[1]
        } else if idx == DIV {
            // println!("DIV {} {}", args[0], args[1]);
            if args[1] == 0.0 {
                retval = 0.0
            } else {
                retval = args[0] / args[1]
            }
        } else if idx == WRITE {
            // println!("WRITE {} {}", args[0], args[1]);
            retval = self.write_rwreg_value(args[0], args[1])
        } else {
            // println!("READ {}", args[0]);
            retval = self.read_reg_value(args[0])
        }
        // println!("result = {}", retval);
        retval
    }

    pub fn gen_rnd_trac_node(&mut self) -> Tree<f32> {
        let func: u8 = self.funcs.gen_rnd_func_all();
        let mut retval: Tree<f32> = Tree::new(func as f32);
        for idx in 1..=arg_count(func) {
            if func == WRITE && idx == 2 {
                retval.push_back(Tree::new(self.rnd_write_idx() as f32));
            } else {
                retval.push_back(Tree::new(self.rnd_read_idx() as f32));
            }
        }
        retval
    }

    pub fn gen_root_node(&mut self) -> Tree<f32> {
        tr(WRITE as f32) / tr(self.rnd_read_idx() as f32) / tr(self.rnd_write_idx() as f32)
    }
}

pub fn simple_init_registry() -> Regs {
    let mut r = Regs::new(5, 5, vec![0.; 5], FuncTr::new());
    r.set_rw_max_value(0.);
    r.set_positive_rw_value(true);
    for i in 0..5 {
        r.write_rwreg_value(i as f32, i as f32);
    }
    for i in 0..5 {
        r.write_roreg_value(i as f32, i);
    }
    r
}

#[cfg(test)]
mod tests {
    use crate::registry::*;

    #[test]
    fn test01() {
        let mut r = simple_init_registry();
        assert_eq!(0 as f32, r.read_reg_value(0 as f32));
        assert_eq!(4 as f32, r.read_reg_value((4) as f32));
        assert_eq!(0 as f32, r.read_reg_value((5) as f32));
        assert_eq!(1 as f32, r.read_reg_value((6) as f32));
        assert_eq!(3 as f32, r.read_reg_value(12.7));
        assert_eq!(2 as f32, r.read_reg_value(16.4));
    }
    #[test]
    fn test02() {
        let mut r = simple_init_registry();
        assert_eq!(4., r.func_call(EQ as f32, &vec![1., 2., 3., 4.]));
        assert_eq!(3., r.func_call(EQ as f32, &vec![1., 1., 3., 4.]));
        assert_eq!(4., r.func_call(GT as f32, &vec![1., 2., 3., 4.]));
        assert_eq!(3., r.func_call(GT as f32, &vec![3., 2., 3., 4.]));
        assert_eq!(5., r.func_call(ADD as f32, &vec![3., 2.]));
        assert_eq!(1., r.func_call(SUB as f32, &vec![3., 2.]));
        assert_eq!(6., r.func_call(MULT as f32, &vec![3., 2.]));
        assert_eq!(3., r.func_call(DIV as f32, &vec![6., 2.]));
        assert_eq!(2., r.func_call(WRITE as f32, &vec![2., 0.]));
        assert_eq!(2., r.read_reg_value(0.));
    }
    #[test]
    fn test03() {
        let mut r = simple_init_registry();
        r.set_sigmoid(true);
        assert_eq!(0.5 as f32, r.read_reg_value(0 as f32));
        assert_eq!(0.9999546, r.read_reg_value((4) as f32));
        assert_eq!(0 as f32, r.read_reg_value((5) as f32));
        assert_eq!(1 as f32, r.read_reg_value((6) as f32));
        assert_eq!(0.99944717 as f32, r.read_reg_value(12.7));
        assert_eq!(2 as f32, r.read_reg_value(16.4));
    }
}
