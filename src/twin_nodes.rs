use crate::*;

pub enum RefreshOrClone {
    Refresh,
    AddTwin,
}
pub fn refresh_or_add_twin(current_frame: u64, old_node_last_frame_touched: u64) -> RefreshOrClone {
    if current_frame == old_node_last_frame_touched {
        // re-adding something that got added in the same frame: making a clone
        return RefreshOrClone::AddTwin;
    } else {
        // re-adding something that was added in an old frame: just a normal refresh
        return RefreshOrClone::Refresh;
    }
}

pub enum TwinCheckResult {
    UpdatedNormal { final_i: NodeI },
    NeedToUpdateTwin { twin_n: u32, twin_key: NodeKey },
}
