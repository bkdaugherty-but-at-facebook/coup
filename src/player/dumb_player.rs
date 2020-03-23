use crate::{Action, GameState, Identity, PlayerID};
use crate::player::traits::Player;

pub struct DumbPlayer {
    // Not necessarily two?
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
    fn choose_action(&self, state: &GameState) -> Action {
        if self.count_coins(state) < 10 {
            Action::Income
        } else {
            // Need to choose player? for coup?
            // choose player after you in order
            // Lol Jank
            for player_id in &state.active_players {
                if player_id != self.who_am_i() {
                    return Action::Coup(player_id.clone());
                }
            }
            panic!("No other players to coup!");
        }
    }
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
        false
    }
    // How do I show this?
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
        false
    }
    // Index in hand to replace
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> usize {
        0
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

    // TODO Deal with errors better
    fn discard_identity(&mut self, state: &GameState) -> Identity {
        let num_cards = self.hand.len();
        if num_cards > 0 {
            // TODO Refactor as util function remove from hand --> Can I make
            // all traits have hand?
            let remove_index = num_cards - 1;
            let removed = self.hand.remove(remove_index);
            return removed;
        } else {
            panic!("Oh God!");
        }
    }
}
