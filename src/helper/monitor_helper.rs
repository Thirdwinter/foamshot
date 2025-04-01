#[derive(Debug, Default, Clone)]
pub struct Monitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale: i32,
}

impl Monitor {
    fn get_right(&self) -> i32 {
        self.x + self.width as i32
    }

    fn get_bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    pub fn is_complete(&self) -> bool {
        !self.name.is_empty() && self.width > 0 && self.height > 0 && self.scale > 0
    }

    // fn get_intersection(
    //     &self,
    //     rect_min_x: i32,
    //     rect_min_y: i32,
    //     rect_max_x: i32,
    //     rect_max_y: i32,
    // ) -> Option<RectIntersection> {
    //     let intersection_min_x = max(self.x, rect_min_x);
    //     let intersection_min_y = max(self.y, rect_min_y);
    //     let intersection_max_x = min(self.get_right(), rect_max_x);
    //     let intersection_max_y = min(self.get_bottom(), rect_max_y);
    //
    //     if intersection_min_x >= intersection_max_x || intersection_min_y >= intersection_max_y {
    //         return None;
    //     }
    //
    //     let relative_min_x = intersection_min_x - self.x;
    //     let relative_min_y = intersection_min_y - self.y;
    //     let width = (intersection_max_x - intersection_min_x) as u32;
    //     let height = (intersection_max_y - intersection_min_y) as u32;
    //
    //     Some(RectIntersection {
    //         monitor_id: self.id,
    //         global_min_x: intersection_min_x,
    //         global_min_y: intersection_min_y,
    //         relative_min_x,
    //         relative_min_y,
    //         width,
    //         height,
    //     })
    // }
}
