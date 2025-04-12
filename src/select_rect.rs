use crate::action::{Action, EditAction};

#[derive(Clone, Debug)]
pub struct SelectRect {
    pub sx: i32,
    pub sy: i32,
    pub ex: i32,
    pub ey: i32,
    // 新增移动状态字段
    move_origin: Option<(f64, f64)>,           // 拖动起始鼠标位置
    rect_origin: Option<(i32, i32, i32, i32)>, // 拖动起始矩形坐标
}

impl SelectRect {
    pub fn new(sx: i32, sy: i32, ex: i32, ey: i32) -> Self {
        Self {
            sx,
            sy,
            ex,
            ey,
            move_origin: None,
            rect_origin: None,
        }
    }
    pub fn edit(&mut self, start_pos: (f64, f64), end_pos: (f64, f64), act: Action) -> Action {
        if act != Action::OnEdit(EditAction::Move) {
            self.move_origin = None;
            self.rect_origin = None;
        }
        match act {
            Action::OnEdit(edit_action) => match edit_action {
                EditAction::Left => {
                    self.sx = end_pos.0 as i32;
                    if self.sx > self.ex {
                        std::mem::swap(&mut self.sx, &mut self.ex);
                        return Action::OnEdit(EditAction::Right);
                    }
                    act
                }
                EditAction::Right => {
                    self.ex = end_pos.0 as i32;
                    if self.ex < self.sx {
                        std::mem::swap(&mut self.sx, &mut self.ex);
                        return Action::OnEdit(EditAction::Left);
                    }
                    act
                }
                EditAction::Top => {
                    self.sy = end_pos.1 as i32;
                    if self.sy > self.ey {
                        std::mem::swap(&mut self.sy, &mut self.ey);
                        return Action::OnEdit(EditAction::Bottom);
                    }
                    act
                }
                EditAction::Bottom => {
                    self.ey = end_pos.1 as i32;
                    if self.ey < self.sy {
                        std::mem::swap(&mut self.sy, &mut self.ey);
                        return Action::OnEdit(EditAction::Top);
                    }
                    act
                }
                EditAction::TopLeft => {
                    self.sx = end_pos.0 as i32;
                    self.sy = end_pos.1 as i32;
                    if self.sx > self.ex {
                        std::mem::swap(&mut self.sx, &mut self.ex);
                        return Action::OnEdit(EditAction::TopRight);
                    }
                    if self.sy > self.ey {
                        std::mem::swap(&mut self.sy, &mut self.ey);
                        return Action::OnEdit(EditAction::BottomLeft);
                    }
                    act
                }
                EditAction::TopRight => {
                    self.ex = end_pos.0 as i32;
                    self.sy = end_pos.1 as i32;
                    if self.ex < self.sx {
                        std::mem::swap(&mut self.sx, &mut self.ex);
                        return Action::OnEdit(EditAction::TopLeft);
                    }
                    if self.sy > self.ey {
                        std::mem::swap(&mut self.sy, &mut self.ey);
                        return Action::OnEdit(EditAction::BottomRight);
                    }
                    return act;
                }
                EditAction::BottomLeft => {
                    self.sx = end_pos.0 as i32;
                    self.ey = end_pos.1 as i32;
                    if self.sx > self.ex {
                        std::mem::swap(&mut self.sx, &mut self.ex);
                        return Action::OnEdit(EditAction::BottomRight);
                    }
                    if self.ey < self.sy {
                        std::mem::swap(&mut self.sy, &mut self.ey);
                        return Action::OnEdit(EditAction::TopLeft);
                    }
                    return act;
                }
                EditAction::BottomRight => {
                    self.ex = end_pos.0 as i32;
                    self.ey = end_pos.1 as i32;
                    if self.ex < self.sx {
                        std::mem::swap(&mut self.sx, &mut self.ex);
                        return Action::OnEdit(EditAction::BottomLeft);
                    }
                    if self.ey < self.sy {
                        std::mem::swap(&mut self.sy, &mut self.ey);
                        return Action::OnEdit(EditAction::TopRight);
                    }
                    return act;
                }
                EditAction::Move => {
                    // 初始化拖动状态
                    if self.move_origin.is_none() {
                        self.move_origin = Some(start_pos);
                        self.rect_origin = Some((self.sx, self.sy, self.ex, self.ey));
                    }

                    // 计算基于初始状态的位移
                    if let (Some((ox, oy)), Some((sx0, sy0, ex0, ey0))) =
                        (self.move_origin, self.rect_origin)
                    {
                        let dx = (end_pos.0 - ox) as i32;
                        let dy = (end_pos.1 - oy) as i32;

                        // 更新坐标保持矩形完整性
                        self.sx = sx0 + dx;
                        self.sy = sy0 + dy;
                        self.ex = ex0 + dx;
                        self.ey = ey0 + dy;
                    }
                    return act;
                }
                _ => act, // EditAction::None => Action::OnEdit(EditAction::None),
            },
            _ => {
                return Action::OnEdit(EditAction::None);
            }
        }
    }

    /// 检测鼠标位置对应的编辑行为
    /// 参数：
    /// * (x, y): 鼠标坐标
    /// * threshold: 临界范围（单位：像素）
    pub fn hit_region(&self, gx: i32, gy: i32, threshold: i32) -> EditAction {
        let (sx, sy, ex, ey) = (self.sx, self.sy, self.ex, self.ey);
        let t = threshold;

        // ================= 顶点检测 =================
        // 左上角检测区域（正方形）
        if (gx >= sx - t) && (gx <= sx + t) && (gy >= sy - t) && (gy <= sy + t) {
            return EditAction::TopLeft;
        }

        // 右上角检测区域
        if (gx >= ex - t) && (gx <= ex + t) && (gy >= sy - t) && (gy <= sy + t) {
            return EditAction::TopRight;
        }

        // 左下角检测区域
        if (gx >= sx - t) && (gx <= sx + t) && (gy >= ey - t) && (gy <= ey + t) {
            return EditAction::BottomLeft;
        }

        // 右下角检测区域
        if (gx >= ex - t) && (gx <= ex + t) && (gy >= ey - t) && (gy <= ey + t) {
            return EditAction::BottomRight;
        }

        // ================= 边检测 =================
        // 左边检测（纵向范围 + 横向阈值）
        if (gx - sx).abs() <= t && gy >= sy && gy <= ey {
            return EditAction::Left;
        }

        // 右边检测
        if (gx - ex).abs() <= t && gy >= sy && gy <= ey {
            return EditAction::Right;
        }

        // 上边检测（横向范围 + 纵向阈值）
        if (gy - sy).abs() <= t && gx >= sx && gx <= ex {
            return EditAction::Top;
        }

        // 下边检测
        if (gy - ey).abs() <= t && gx >= sx && gx <= ex {
            return EditAction::Bottom;
        }

        // ================= 移动检测 =================
        // 如果在内区则返回移动操作
        if gx > sx && gx < ex && gy > sy && gy < ey {
            return EditAction::Move;
        }

        EditAction::None
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubRect {
    pub monitor_id: usize,
    pub relative_min_x: i32,
    pub relative_min_y: i32,
    pub width: i32,
    pub height: i32,
}
impl SubRect {
    pub fn new(id: usize, x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            monitor_id: id,
            relative_min_x: x,
            relative_min_y: y,
            width: w,
            height: h,
        }
    }
}
