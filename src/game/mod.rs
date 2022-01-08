use std::collections::HashMap;

type CellList = HashMap<(i32, i32), bool>;

const OFFSETS: [(i32, i32); 8] = [
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

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    pub fn new() -> Game {
        let mut list = CellList::new();
        let size = 0..10;
        list.insert((0, 0), true);
        list.insert((1, 0), true);
        list.insert((1, 1), true);
        list.insert((2, 1), true);
        for i in size.clone() {
            for j in size.clone() {
                match list.get(&(i, j)) {
                    Some(_) => (),
                    None => {
                        list.insert((i, j), false);
                    }
                };
            }
        }
        Game { list }
    }

    pub fn make_list(&self) -> Vec<f32> {
        let _instance = crate::model::Instance {
            position: na::Point3::new(0.0, 0.0, 0.0),
            rotation: na::UnitQuaternion::from_axis_angle(&na::Vector3::x_axis(), 0.0),
        };
        let _instance2 = crate::model::Instance {
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
        for (k, v) in self.list.iter() {
            let neighbors = self.get_neighbors(*k);
            if !(2..=3).contains(&neighbors) {
                changes.push((k.0, k.1, false));
            }

            if neighbors == 3 && !*v {
                changes.push((k.0, k.1, true));
            }

            if neighbors == 2 || neighbors == 3 {
                //nothing
            }
        }

        for c in changes {
            self.list.insert((c.0, c.1), c.2);
        }
    }

    fn get_neighbors(&self, cell: (i32, i32)) -> u32 {
        let mut count: u32 = 0;
        for (x, y) in OFFSETS {
            if let Some(alive) = self.list.get(&(cell.0 + x, cell.1 + y)) {
                if *alive {
                    count += 1;
                }
            }
        }

        count
    }
}
