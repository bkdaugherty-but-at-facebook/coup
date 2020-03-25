use crate::PlayerID;

#[derive(Debug, Clone)]
pub enum Action {
    Income,
    ForeignAid,
    Tax,
    Assassinate(PlayerID),
    Coup(PlayerID),
    Steal(PlayerID),
    Exchange,
    BlockForeignAid,
    BlockAssassination,
    BlockStealCaptain,
    BlockStealAmbassador,
}

impl Action {
    // Dependent on id of target
    pub fn blockable(&self, id: &PlayerID) -> Option<Vec<Action>> {
        match self {
            Action::Income
            | Action::Coup(..)
            | Action::Exchange
            | Action::Tax
            | Action::BlockForeignAid
            | Action::BlockAssassination
            | Action::BlockStealCaptain
            | Action::BlockStealAmbassador => None,

            // Can only block if they target you
            Action::Assassinate(target) => match target == id {
                true => Some(vec![Action::BlockAssassination]),
                false => None,
            },
            Action::Steal(target) => match target == id {
                true => Some(vec![
                    Action::BlockStealCaptain,
                    Action::BlockStealAmbassador,
                ]),
		false => None,
            },
            Action::ForeignAid => Some(vec![Action::BlockForeignAid]),
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
