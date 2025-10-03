use std::collections::{HashMap, HashSet};

use my_cfg::{Arena, Cfg, Idx, Local, Place, Value};

pub fn remove_unneeded_locals(cfg: &mut Cfg) {
    fn is_local_needed(cfg: &Cfg, local: Idx<Local>) -> bool {
        let return_local = Idx::from_usize(0);
        let return_local = || my_cfg::Value::Place(Place::Local(return_local));
        cfg.bb.iter().any(|(_, bb)| {
            bb.stmts.iter().any(|stmt| match stmt {
                my_cfg::Stmt::Assign { place, value } => {
                    if place
                        .replace_local(local, return_local())
                        .as_place()
                        .unwrap()
                        != place
                    {
                        return true;
                    }
                    if value.has_local(local) {
                        return true;
                    }
                    false
                }
            })
        })
    }

    let mut new_locals = Arena::default();
    let mut map = vec![];
    for (local, data) in cfg.locals.iter() {
        if is_local_needed(cfg, local) {
            let new_local = new_locals.alloc(data.clone());
            map.push((local, new_local));
        }
    }

    cfg.locals = new_locals;

    for (old_local, new_local) in map {
        for (_, bb) in cfg.bb.iter_mut() {
            for stmt in &mut bb.stmts {
                match stmt {
                    my_cfg::Stmt::Assign { place, value } => {
                        *place = place
                            .replace_local(old_local, Value::from_local(new_local))
                            .as_place()
                            .unwrap()
                            .clone();
                        *value = value.replace_local(old_local, Value::from_local(new_local));
                    }
                }
            }
            match bb.terminator.as_mut().unwrap() {
                my_cfg::Terminator::Return | my_cfg::Terminator::Goto { bb: _ } => {},
                my_cfg::Terminator::If { cond, then: _, else_: _ } => {
                    *cond = cond.replace_local(old_local, Value::from_local(new_local));
                }
            }
        }
    }
}

fn is_local_value_read_by_block(l: Idx<Local>, cfg: &Cfg) -> bool {
    cfg.bb.iter().any(|(_, bb)| {
        for stmt in &bb.stmts {
            match stmt {
                my_cfg::Stmt::Assign { place, value } => {
                    if value.has_local(l) {
                        return true;
                    }
                    if place.as_local() == Some(l) {
                        return false;
                    }
                }
            }
        }
        match bb.terminator() {
            my_cfg::Terminator::Return | my_cfg::Terminator::Goto { bb: _ } => false,
            my_cfg::Terminator::If {
                cond,
                then: _,
                else_: _,
            } => cond.has_local(l),
        }
    })
}

pub fn remove_unneeded_assigns(cfg: &mut Cfg) {
    let used_locals: HashSet<_> = cfg
        .locals
        .iter()
        .filter(|(l, _)| is_local_value_read_by_block(*l, cfg))
        .map(|x| x.0)
        .collect();
    for (_, bb) in cfg.bb.iter_mut() {
        let mut indexes_to_remove = vec![];
        for index in 0..bb.stmts.len() {
            let stmt = bb.stmts[index].clone();
            match stmt {
                my_cfg::Stmt::Assign {
                    place,
                    value: my_value,
                } => {
                    let Some(l) = place.as_local() else {
                        continue;
                    };
                    if used_locals.contains(&l) {
                        continue;
                    }
                    indexes_to_remove.push(index);
                    let mut finished = false;
                    for rest in &mut bb.stmts[index + 1..] {
                        match rest {
                            my_cfg::Stmt::Assign { place, value } => {
                                if place.as_local() == Some(l) {
                                    finished = true;
                                    break;
                                }
                                *place = place
                                    .replace_local(l, my_value.clone())
                                    .as_place()
                                    .unwrap()
                                    .clone();
                                *value = value.replace_local(l, my_value.clone());
                            }
                        }
                    }
                    if !finished {
                        match bb.terminator.as_mut().unwrap() {
                            my_cfg::Terminator::Return | my_cfg::Terminator::Goto { bb: _ } => (),
                            my_cfg::Terminator::If {
                                cond,
                                then: _,
                                else_: _,
                            } => {
                                *cond = cond.replace_local(l, my_value.clone());
                            }
                        }
                    }
                }
            }
        }
        for i in indexes_to_remove.into_iter().rev() {
            bb.stmts.remove(i);
        }
    }
}
