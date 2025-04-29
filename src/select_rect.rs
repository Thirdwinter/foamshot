//! INFO: Provides data processing for the dragged rectangle during user interaction

use crate::action::{Action, EditAction};

/// 默认的检测范围值
pub const THRESHOLD: i32 = 15;

#[derive(Clone, Debug)]
/// 表示当前选择的矩形区域
pub struct SelectRect {
    pub sx: i32,
    pub sy: i32,
    pub ex: i32,
    pub ey: i32,

    // 拖动起始鼠标位置
    move_origin: Option<(f64, f64)>,
    // 拖动起始矩形坐标
    rect_origin: Option<(i32, i32, i32, i32)>,
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
    /// NOTE: 需要返回新的Action
    pub fn edit(&mut self, start_pos: (f64, f64), end_pos: (f64, f64), act: Action) -> Action {
        // 检查是否需要重置移动状态：如果不是移动操作，或者是新的移动开始（start_pos 变化）
        let should_reset = match act {
            Action::OnEdit(EditAction::Move) => {
                if let Some(origin) = self.move_origin {
                    // 如果是新的移动操作（起始位置不同）
                    // start_pos 是每次光标按下确定，因此一次连续的按下-拖动-松开过程中不变，这里返回false
                    // 如果重新按下，这里传入的start_pos会被更新，因此需要清除上一次移动的缓存信息
                    start_pos != origin
                } else {
                    true
                }
            }
            _ => true,
        };

        if should_reset {
            self.move_origin = None;
            self.rect_origin = None;
        }
        match act {
            Action::OnEdit(edit_action) => match edit_action {
                EditAction::Move => {
                    // 首次开始移动时，记录初始状态
                    if self.move_origin.is_none() {
                        self.move_origin = Some(start_pos);
                        self.rect_origin = Some((self.sx, self.sy, self.ex, self.ey));
                    }

                    // 基于初始状态和当前鼠标位置计算新位置
                    if let (Some(origin_pos), Some(origin_rect)) =
                        (self.move_origin, self.rect_origin)
                    {
                        // 计算相对于拖动开始位置的总位移
                        let dx = (end_pos.0 - origin_pos.0) as i32;
                        let dy = (end_pos.1 - origin_pos.1) as i32;

                        // 基于初始矩形位置计算新位置
                        self.sx = origin_rect.0 + dx;
                        self.sy = origin_rect.1 + dy;
                        self.ex = origin_rect.2 + dx;
                        self.ey = origin_rect.3 + dy;
                    }

                    Action::OnEdit(EditAction::Move)
                }

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
                    act
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
                    act
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
                    act
                }
                _ => act,
            },
            _ => Action::OnEdit(EditAction::None),
        }
    }

    /// 检测鼠标位置对应的编辑行为
    /// threshold: 临界范围（单位：像素）
    pub fn hit_region(&self, gx: i32, gy: i32, threshold: i32) -> EditAction {
        let (sx, sy, ex, ey) = (self.sx, self.sy, self.ex, self.ey);
        let t = threshold;

        // 左上角检测区域
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

        // 左边检测
        if (gx - sx).abs() <= t && gy >= sy && gy <= ey {
            return EditAction::Left;
        }

        // 右边检测
        if (gx - ex).abs() <= t && gy >= sy && gy <= ey {
            return EditAction::Right;
        }

        // 上边检测
        if (gy - sy).abs() <= t && gx >= sx && gx <= ex {
            return EditAction::Top;
        }

        // 下边检测
        if (gy - ey).abs() <= t && gx >= sx && gx <= ex {
            return EditAction::Bottom;
        }

        // 如果在矩形内则返回移动操作
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
