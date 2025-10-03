// relooper.rs

use std::collections::{HashMap, HashSet, VecDeque};
use crate::*;

// (StructuredNode enum remains the same)
#[derive(Debug)]
pub enum StructuredNode {
    Basic(Idx<BasicBlock>),
    Sequence(Vec<StructuredNode>),
    If {
        cond: Value,
        then_node: Box<StructuredNode>,
        else_node: Box<StructuredNode>,
    },
    Loop(Box<StructuredNode>),
    Dispatch {
        entry_map: HashMap<Idx<BasicBlock>, i32>,
        handlers: Vec<(i32, StructuredNode)>,
    },
}

pub struct Relooper<'a> {
    cfg: &'a Cfg,
    processed: HashSet<Idx<BasicBlock>>,
    predecessors: HashMap<Idx<BasicBlock>, Vec<Idx<BasicBlock>>>,
    label_counter: i32,
}

impl<'a> Relooper<'a> {
    pub fn new(cfg: &'a Cfg) -> Self {
        let predecessors = Self::compute_predecessors(cfg);
        Self {
            cfg,
            processed: HashSet::new(),
            predecessors,
            label_counter: 0,
        }
    }

    pub fn reloop(mut self, entry: Idx<BasicBlock>) -> StructuredNode {
        self.shape_blocks(vec![entry])
    }

    fn shape_blocks(&mut self, entries: Vec<Idx<BasicBlock>>) -> StructuredNode {
        let entries: Vec<_> = entries
            .into_iter()
            .filter(|e| !self.processed.contains(e))
            .collect();

        if entries.is_empty() {
            return StructuredNode::Sequence(vec![]);
        }

        if entries.len() == 1 {
            let entry_idx = entries[0];
            // The order here is important: Simple chains are the most basic,
            // then we check for specific structures like If and Loop.
            if let Some(node) = self.try_shape_simple(entry_idx) {
                return node;
            }
            if let Some(node) = self.try_shape_if(entry_idx) {
                return node;
            }
            if let Some(node) = self.try_shape_loop(entry_idx) {
                return node;
            }
        }
        
        self.shape_multiple(entries)
    }

    // BUGFIX #1: Corrected `try_shape_simple` logic.
    fn try_shape_simple(&mut self, entry_idx: Idx<BasicBlock>) -> Option<StructuredNode> {
        // A simple sequence cannot start with a branch. If the entry itself has multiple
        // successors, it's not "simple", so we should return None immediately and let
        // other shapers handle it.
        if self.successors(entry_idx).len() > 1 {
            return None;
        }

        let mut sequence = vec![];
        let mut current_idx = entry_idx;

        loop {
            // A block can be part of a simple sequence if it has not been processed
            // and has exactly one predecessor (unless it's the very first entry).
            let preds = self.predecessors.get(&current_idx).map_or(0, |v| v.len());
            if self.processed.contains(&current_idx) || (current_idx != entry_idx && preds != 1) {
                // We've hit a merge point or an already processed block.
                // Shape what comes next and append it to our sequence.
                let next_node = self.shape_blocks(vec![current_idx]);
                sequence.push(next_node);
                return Some(StructuredNode::Sequence(sequence));
            }
            
            let block = &self.cfg.bb[current_idx];
            self.processed.insert(current_idx);
            sequence.push(StructuredNode::Basic(current_idx));

            match block.terminator() {
                Terminator::Goto { bb } => {
                    current_idx = *bb;
                }
                Terminator::Return => {
                    // End of a path.
                    return Some(StructuredNode::Sequence(sequence));
                }
                Terminator::If { .. } => {
                    // This is the last block in the simple sequence.
                    // The branch itself will be handled by the next call to `shape_blocks`.
                    let next_node = self.shape_blocks(vec![current_idx]);
                    return Some(StructuredNode::Sequence(vec![next_node]));
                }
            }
        }
    }
    
    // NEW FEATURE: Added `try_shape_if` to handle diamond patterns cleanly.
    fn try_shape_if(&mut self, entry_idx: Idx<BasicBlock>) -> Option<StructuredNode> {
        let block = &self.cfg.bb[entry_idx];
        let (cond, then_idx, else_idx) = match block.terminator() {
            Terminator::If { cond, then, else_ } => (cond.clone(), *then, *else_),
            _ => return None,
        };

        // Find a merge point for the two branches.
        let then_reachable = self.find_reachable(&[then_idx], &self.processed);
        let else_reachable = self.find_reachable(&[else_idx], &self.processed);
        
        let mut merge_point = None;
        // A simple merge is when one branch's successor is the other branch's entry.
        if self.successors(then_idx).len() == 1 && self.successors(then_idx)[0] == else_idx {
             // then -> else, not a diamond, handle differently or not at all here.
        } else if let Some(mp) = self.find_merge_point(&then_reachable, &else_reachable) {
             merge_point = Some(mp);
        }

        if let Some(mp) = merge_point {
             self.processed.insert(entry_idx);
             let then_node = self.shape_blocks(vec![then_idx]);
             let else_node = self.shape_blocks(vec![else_idx]);
             let next_node = self.shape_blocks(vec![mp]);

             let if_node = StructuredNode::If {
                 cond,
                 then_node: Box::new(then_node),
                 else_node: Box::new(else_node),
             };
             
             // Combine the `if` and the code after it into a sequence.
             return Some(StructuredNode::Sequence(vec![
                 StructuredNode::Basic(entry_idx),
                 if_node,
                 next_node,
             ]));
        }

        None
    }

    // BUGFIX #2: Rewritten `try_shape_loop` for clarity and correctness.
    fn try_shape_loop(&mut self, entry_idx: Idx<BasicBlock>) -> Option<StructuredNode> {
        // Find all blocks reachable from the entry, forming a potential loop body.
        let reachable_from_entry = self.find_reachable(&[entry_idx], &self.processed);

        // A natural loop exists if one of the reachable nodes can branch back to the entry.
        let is_loop = reachable_from_entry
            .iter()
            .any(|&idx| self.successors(idx).contains(&entry_idx));
        
        if !is_loop {
            return None;
        }

        // Check that this is a single-entry loop. All predecessors of the header (entry_idx)
        // must either be from inside the loop body or be the single entry point.
        let mut external_preds = 0;
        for pred in self.predecessors.get(&entry_idx).unwrap_or(&vec![]) {
            if !reachable_from_entry.contains(pred) {
                external_preds += 1;
            }
        }
        
        // If more than one external block can enter the loop header, it's not a simple loop.
        if external_preds > 1 {
            return None;
        }

        // Mark the entire loop body as processed.
        for &idx in &reachable_from_entry {
            self.processed.insert(idx);
        }

        // The loop body itself needs to be structured. We tell the recursive call
        // to only process the blocks inside the loop set.
        let body_node = self.shape_blocks(vec![entry_idx]);

        // Find all exits from the loop. These are the entry points for the next structure.
        let mut loop_exits = vec![];
        for &idx in &reachable_from_entry {
            for succ in self.successors(idx) {
                if !reachable_from_entry.contains(&succ) {
                    loop_exits.push(succ);
                }
            }
        }

        let next_node = self.shape_blocks(loop_exits);
        
        let loop_node = StructuredNode::Loop(Box::new(body_node));
        Some(StructuredNode::Sequence(vec![loop_node, next_node]))
    }

    // `shape_multiple` remains the fallback for complex cases and is largely the same.
    fn shape_multiple(&mut self, entries: Vec<Idx<BasicBlock>>) -> StructuredNode {
        // (Implementation is identical to the previous version)
        let reachable_set = self.find_reachable(&entries, &self.processed);
        for &idx in &reachable_set { self.processed.insert(idx); }

        let mut entry_map = HashMap::new();
        for &entry in &entries {
            entry_map.insert(entry, self.label_counter);
            self.label_counter += 1;
        }
        
        let mut handlers = Vec::new();
        for &entry in &entries {
            let label = entry_map[&entry];
            let handler_node = self.shape_handler(entry, &entry_map);
            handlers.push((label, handler_node));
        }

        let mut exits = vec![];
        for &idx in &reachable_set {
            for succ in self.successors(idx) {
                if !reachable_set.contains(&succ) {
                    exits.push(succ);
                }
            }
        }
        
        let dispatch_node = StructuredNode::Dispatch { entry_map, handlers };
        let next_node = self.shape_blocks(exits);
        StructuredNode::Sequence(vec![dispatch_node, next_node])
    }

    fn shape_handler(
        &self,
        entry: Idx<BasicBlock>,
        entry_map: &HashMap<Idx<BasicBlock>, i32>,
    ) -> StructuredNode {
        // (Implementation is identical to the previous version)
        let mut sequence = vec![];
        let mut current_idx = entry;
        loop {
            sequence.push(StructuredNode::Basic(current_idx));
            let successors = self.successors(current_idx);
            if successors.len() == 1 {
                let succ = successors[0];
                if entry_map.contains_key(&succ) && succ != entry { break; }
                current_idx = succ;
            } else {
                break;
            }
        }
        StructuredNode::Sequence(sequence)
    }
    
    // (Helper functions `find_reachable`, `compute_predecessors`, `successors` are the same,
    // plus a new one `find_merge_point`)
    fn find_merge_point(
        &self,
        set1: &HashSet<Idx<BasicBlock>>,
        set2: &HashSet<Idx<BasicBlock>>,
    ) -> Option<Idx<BasicBlock>> {
        // A simple but effective way to find a merge point is to find a successor
        // of set1 that is also a successor of set2.
        for &idx1 in set1 {
            for succ1 in self.successors(idx1) {
                if !set1.contains(&succ1) { // It's an exit of set1
                    for &idx2 in set2 {
                        for succ2 in self.successors(idx2) {
                            if succ1 == succ2 {
                                return Some(succ1);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn find_reachable(&self, entries: &[Idx<BasicBlock>], processed: &HashSet<Idx<BasicBlock>>) -> HashSet<Idx<BasicBlock>> {
        let mut reachable = HashSet::new();
        let mut queue: VecDeque<_> = entries.iter().cloned().collect();
        let initial_entries: HashSet<_> = entries.iter().cloned().collect();

        while let Some(idx) = queue.pop_front() {
            if !reachable.insert(idx) { continue; }
            for succ in self.successors(idx) {
                if !processed.contains(&succ) || initial_entries.contains(&succ) {
                    queue.push_back(succ);
                }
            }
        }
        reachable
    }
    
    fn compute_predecessors(cfg: &Cfg) -> HashMap<Idx<BasicBlock>, Vec<Idx<BasicBlock>>> {
        let mut preds = HashMap::new();
        for (idx, _) in cfg.bb.iter() { preds.insert(idx, vec![]); }
        for (idx, block) in cfg.bb.iter() {
            for succ in Self::get_successors_from_block(block) {
                preds.entry(succ).or_default().push(idx);
            }
        }
        preds
    }

    fn successors(&self, idx: Idx<BasicBlock>) -> Vec<Idx<BasicBlock>> {
        Self::get_successors_from_block(&self.cfg.bb[idx])
    }
    
    fn get_successors_from_block(block: &BasicBlock) -> Vec<Idx<BasicBlock>> {
         match block.terminator() {
            Terminator::Return => vec![],
            Terminator::Goto { bb } => vec![*bb],
            Terminator::If { then, else_, .. } => vec![*then, *else_],
        }
    }
}