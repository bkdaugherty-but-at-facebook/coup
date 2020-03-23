use crate::PlayerID;

#[derive(Debug)]
pub enum Action {
    Income,
    ForeignAid,
    Tax,
    Assassinate(PlayerID),
    Coup(PlayerID),
    Exchange,
    // BlockForeignAid
    // BlockAssassination
}

impl Action {
    // Dependent on id of target
    pub fn blockable(&self, id: &PlayerID) -> bool {
        match self {
            Action::Income | Action::Coup(..) | Action::Exchange | Action::Tax => false,
            // Can only block if they assassinate you
            Action::Assassinate(target) => target == id,
            Action::ForeignAid => true,
        }
    }
    // Defines if an action is challengable
    pub fn challengable(&self) -> bool {
        match self {
            Action::Income | Action::ForeignAid | Action::Coup(..) => false,
            _ => true,
        }
    }
}

