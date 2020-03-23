use crate::{Action, GameState, Identity, PlayerID};
use std::convert::TryInto;
use std::mem;

const MAX_CARDS: u8 = 2;

pub trait Player {
    fn choose_action(&self, state: &GameState) -> Action;
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> usize;

    // Utility functions on player state
    fn get_hand(&self) -> Vec<Identity>;
    fn set_hand(&mut self, hand: Vec<Identity>);
    fn who_am_i(&self) -> &PlayerID;
    fn discard_identity(&mut self, state: &GameState) -> Identity;

    // Start built-in functions
    fn replace_card(&mut self, to_replace: usize, card: Identity) {
        let mut hand = self.get_hand();
        mem::replace(&mut hand[to_replace], card.clone());
        self.set_hand(hand);
    }

    fn hand_full(&self) -> bool {
        self.get_hand().len() >= MAX_CARDS.try_into().unwrap()
    }

    fn count_coins(&self, state: &GameState) -> u8 {
        let player_state = state.player_states.get(self.who_am_i()).unwrap();
        player_state.num_coins.clone()
    }

    fn take_card(&mut self, state: &GameState, card: Identity) {
        // Yeah this is silly
        if self.hand_full() {
            let to_replace = self.choose_card_to_replace(state, &card);
            self.replace_card(to_replace, card);
        } else {
            let mut hand = self.get_hand();
            hand.push(card);
            self.set_hand(hand);
        }
    }
}
