use std::cell::RefCell;
use crate::context::StackValue;

pub struct FrameVars {
    vars: Vec<StackValue>,
    frame_offset: Vec<usize>,
}

impl FrameVars {
    pub fn new() -> Self {
        FrameVars {
            vars: Vec::with_capacity(256),
            frame_offset: vec![0],
        }
    }
    
    pub fn push(&mut self) {
        self.frame_offset.push(self.vars.len());
    }
    
    pub fn pop(&mut self) {
        self.frame_offset.pop();
    }
    
    fn add_if_needed(&mut self, position: usize) {
        if position < self.vars.len() {
            return
        }
        while position >= self.vars.len() {
            self.vars.push(StackValue::Blank);
        }
    }
    
    pub fn get(&self, index: usize) -> Option<&StackValue> {
        let Some(start) = self.frame_offset.last().cloned() else {
            unreachable!("We should always have at least one frame")
        };
        self.vars.get(start + index)
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut StackValue> {
        let Some(start) = self.frame_offset.last_mut().cloned() else {
            unreachable!("We should always have at least one frame")
        };
        self.add_if_needed(start + index);
        self.vars.get_mut(start + index)
    }
    
    pub fn set(&mut self, index: usize, value: StackValue) {
        let Some(start) = self.frame_offset.last().cloned() else {
            unreachable!("We should always have at least one frame")
        };
        self.add_if_needed(start + index);
        self.vars[start + index] = value;
    }
}

impl std::ops::Index<usize> for FrameVars {
    type Output = StackValue;
    fn index(&self, index: usize) -> &Self::Output {
        let Some(value) = self.get(index) else {
            panic!("FrameVars::index out of bounds");
        };
        value
    }
}

impl std::ops::IndexMut<usize> for FrameVars {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let Some(value) = self.get_mut(index) else {
            panic!("FrameVars::index out of bounds: {}", index);
        };
        value
    }
}