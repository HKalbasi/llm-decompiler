use std::collections::HashMap;

use my_cfg::{BasicBlock, Cfg, Idx, Local, Place, Stmt, Value};
use z3::ast::{Array, BV};

struct Z3CfgState {
    cfg: Cfg,
    local_addrs: HashMap<Idx<Local>, i64>,
    memory: Array,
}

fn z3_new_general_memory() -> Array {
    let memory = Array::fresh_const("memory", &z3::Sort::bitvector(64), &z3::Sort::bitvector(8));
    memory
}

impl Z3CfgState {
    fn new(cfg: Cfg) -> Self {
        let mut addr = 1000;
        let mut local_addrs = HashMap::new();
        for (l, data) in cfg.locals.iter() {
            local_addrs.insert(l, addr);
            addr += data.ty.size() as i64;
        }
        Self {
            cfg,
            local_addrs,
            memory: z3_new_general_memory(),
        }
    }

    fn read_memory(&self, addr: BV, size_bytes: u32) -> BV {
        let mut r = self.memory.select(&addr).as_bv().unwrap();
        for i in 1..size_bytes {
            r = r.concat(self.memory.select(&addr.bvadd(i)).as_bv().unwrap());
        }
        r
    }

    fn write_memory(&mut self, addr: BV, value: BV, size_bytes: u32) {
        for i in 0..size_bytes {
            self.memory = self
                .memory
                .store(&addr.bvadd(i), &value.extract(i * 8 + 7, i * 8));
        }
    }

    fn z3_of_place_addr(&self, place: &Place) -> BV {
        match place {
            Place::Local(idx) => BV::from_i64(self.local_addrs[idx], 64),
            Place::Deref(place) => self.read_memory(self.z3_of_place_addr(place), 8),
            Place::Offset(place, value) => {
                let size_l = place.ty(&self.cfg).size() as u32 * 8;
                let size_r = value.ty(&self.cfg).size() as u32 * 8;
                let size = size_l.max(size_r);
                self.z3_of_place_addr(place)
                    .sign_ext(size - size_l)
                    .bvadd(self.z3_of_value(value).sign_ext(size - size_r))
            }
        }
    }

    fn z3_of_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { place, value } => {
                let size = place.ty(&self.cfg).size();
                let addr = self.z3_of_place_addr(place);
                let value = self.z3_of_value(value);
                self.write_memory(addr, value, size as u32);
            }
        }
    }

    fn z3_of_value(&self, value: &Value) -> BV {
        let r = match value {
            Value::Place(place) => {
                let size = place.ty(&self.cfg).size();
                let addr = self.z3_of_place_addr(place);
                self.read_memory(addr, size as _)
            }
            Value::Literal(i) => BV::from_i64(*i as i64, 32),
            Value::Binop(l, binop, r) => {
                let size_l = l.ty(&self.cfg).size() as u32 * 8;
                let size_r = r.ty(&self.cfg).size() as u32 * 8;

                let mut l = self.z3_of_value(l);
                let mut r = self.z3_of_value(r);

                if size_l > size_r {
                    r = r.sign_ext(size_l - size_r);
                }
                if size_r > size_l {
                    l = l.sign_ext(size_r - size_l);
                }

                match binop {
                    my_cfg::Binop::Add => l.bvadd(r),
                    my_cfg::Binop::Sub => l.bvsub(r),
                    my_cfg::Binop::Mul => l.bvmul(r),
                    my_cfg::Binop::Div => l.bvsdiv(r),
                    my_cfg::Binop::Lt => todo!(),
                    my_cfg::Binop::Le => todo!(),
                }
            }
        };
        dbg!(&r);
        BV::fresh_const("value", r.get_size())
    }
}

pub fn z3_of_bb_stmts(bb: &BasicBlock, cfg: &Cfg) {
    let mut this = Z3CfgState::new(cfg.clone());
    for x in &bb.stmts {
        this.z3_of_stmt(x);
    }
    dbg!(this.memory);
}
