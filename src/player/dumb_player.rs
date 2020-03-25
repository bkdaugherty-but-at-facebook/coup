use crate::{Action, GameState, Identity, PlayerID};
use crate::player::traits::Player;
use anyhow::Result;

pub struct DumbPlayer {
    id: PlayerID,
    hand: Vec<Identity>,
}

impl DumbPlayer {
    pub fn new(id: PlayerID) -> Self {
        let hand = Vec::new();
        DumbPlayer { id, hand }
    }
}

impl Player for DumbPlayer {
    fn choose_action(&self, _state: &GameState) -> Action {
        Action::Income
    }
    fn will_challenge(&self, _state: &GameState, _player_id: &PlayerID, _action: &Action) -> bool {
        false
    }
    fn will_block(&self, _state: &GameState, _player_id: &PlayerID, _action: &Action) -> Option<Action> {
	None
    }
    // Index in hand to replace
    fn choose_card_to_replace(&self, _state: &GameState, _card: &Identity) -> Option<usize> {
	None
    }

    fn choose_card_to_lose(&self, _state: &GameState) -> usize {
	0
    }

    fn choose_forced_coup(&self, state: &GameState) -> PlayerID {
	for player_id in &state.active_players {
            if player_id != self.who_am_i() {
                return player_id.clone();
            }
        }
        panic!("No other players to coup!");
    }
    
    fn get_hand(&self) -> Vec<Identity> {
        self.hand.iter().cloned().collect()
    }

    fn set_hand(&mut self, hand: Vec<Identity>) {
        self.hand = hand;
    }

    fn who_am_i(&self) -> &PlayerID {
        &self.id
    }

    
}
