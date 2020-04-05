use crate::PlayerID;
use std::fmt;


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


impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	let to_write = match self {
	    Action::Income => "Income".to_string(),
	    Action::ForeignAid => "Foreign Aid".to_string(),
	    Action::Tax => "Tax".to_string(),
	    Action::Assassinate(..) => "Assassinate".to_string(),
	    Action::Coup(..) => "Coup".to_string(),
	    Action::Steal(..) => "Steal".to_string(),
	    Action::Exchange => "Exchange".to_string(),
	    Action::BlockForeignAid => "Block Foreign Aid".to_string(),
	    Action::BlockAssassination => "Contess".to_string(),
	    Action::BlockStealCaptain => "Block as Captain".to_string(),
	    Action::BlockStealAmbassador => "Block as Ambassador".to_string(),
	    _ => format!("{:?}", self)
	};
        write!(f, "{}", to_write)
    }
}




