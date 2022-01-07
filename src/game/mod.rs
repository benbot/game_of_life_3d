use std::{cell::Cell, collections::HashMap, hash::Hash};

type CellList = HashMap<(i32, i32), bool>;

const offsets: [(i32, i32); 8] = [
    //right
    (1, 0),
    //left
    (-1, 0),
    //top
    (0, 1),
    //bottom
    (0, -1),
    //top left,
    (-1, 1),
    //top right,
    (1, 1),
    //bottom left,
    (-1, -1),
    //bottom right
    (1, -1),
];

#[derive(Debug)]
pub struct Game {
    pub list: CellList,
}

impl Game {
    pub fn new() -> Game {
        let mut list = CellList::new();
        for i in 0..10 {
            for j in 0..10 {
                list.insert((i, j), false);
            }
        }
        Game {
            list: CellList::new(),
        }
    }

    pub fn make_list(&self) -> Vec<f32> {
        let instance = crate::model::Instance {
            position: na::Point3::new(0.0, 0.0, 0.0),
            rotation: na::UnitQuaternion::from_axis_angle(&na::Vector3::x_axis(), 0.0),
        };
        let instance2 = crate::model::Instance {
            position: na::Point3::new(1.0, 1.0, 0.0),
            rotation: na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
        };

        let mut instances = vec![];

        for (k, v) in self.list.iter() {
            if *v {
                instances.push(crate::model::Instance {
                    position: na::Point3::new(k.0 as f32, k.1 as f32, 0.0),
                    rotation: na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
                })
            }
        }

        let mut out = vec![];

        for i in instances {
            out.append(&mut i.to_raw());
        }

        out
    }

    pub fn update(&mut self) {
        let mut changes = vec![];
        for (k, _v) in self.list.iter() {
            if get_neighbors(&self.list, (k.0, k.1)) > 1 {
                changes.push((k.0, k.1, true));
            }
        }

        for c in changes {
            self.list.insert((c.0, c.1), c.2);
        }
    }
}

fn get_neighbors(list: &CellList, cell: (i32, i32)) -> u32 {
    let mut count = 0;
    for (x, y) in offsets {}
    todo!();
}
