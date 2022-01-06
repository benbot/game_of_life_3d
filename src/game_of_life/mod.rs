use std::collections::LinkedList;

#[derive(Debug)]
pub struct Game {
    boxys: LinkedList<na::Vector3<u32>>,
}

impl Game {
    pub fn to_instance_array(&self) {}
}
