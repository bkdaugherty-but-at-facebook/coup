use crate::{Action, GameState, Identity, PlayerID};
use std::convert::TryInto;
use std::mem;
use anyhow::{anyhow, Result};

const MAX_CARDS: u8 = 2;

pub trait Player {
    fn choose_action(&self, state: &GameState) -> Action;
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> Option<Action>;
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> Option<usize>;
    fn choose_card_to_lose(&self, state: &GameState) -> usize;
    fn choose_forced_coup(&self, state: &GameState) -> PlayerID;
    
    // Utility functions on player state
    fn get_hand(&self) -> Vec<Identity>;
    fn set_hand(&mut self, hand: Vec<Identity>);
    fn who_am_i(&self) -> &PlayerID;

    // Start built-in functions
    fn lose_challenge(&mut self, state: &GameState) -> Identity {
	// TODO handle user errors
	self.lose_life(state)
    }

    fn lose_life(&mut self, state: &GameState) -> Identity {
	// TODO handle user errors --> Don't require choice if only one card to lose
	let index = self.choose_card_to_lose(state);
	match self.discard(index) {
	    Ok(identity) => identity,
	    Err(e) => self.lose_life(state),
	}
    }

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
	    if let Some(to_replace) = self.choose_card_to_replace(state, &card) {
		self.replace_card(to_replace, card);
	    }
        } else {
            let mut hand = self.get_hand();
            hand.push(card);
            self.set_hand(hand);
        }
    }
    
    fn discard(&mut self, index: usize) -> Result<Identity> {
	let mut hand = self.get_hand();
	if index < hand.len() {
	    let card = hand.remove(index);
	    self.set_hand(hand);
	    Ok(card)
	} else {
	    Err(anyhow!("Index out of range - {} of hand {}", index, hand.len()))
	}
    }

    // I would prefer this translation be in action but this is more flexible for
    // embezzlement
    // Maybe not actually
    fn can_do_action(&self, action: &Action) -> bool {
	match action {
	    Action::Income | Action::ForeignAid | Action::Coup(..) => true,
	    Action::Assassinate(..) => self.has_identity(Identity::Assassin),
	    Action::Tax => self.has_identity(Identity::Duke),
	    Action::Exchange => self.has_identity(Identity::Ambassador),
	    Action::BlockForeignAid => self.has_identity(Identity::Duke),
	    Action::BlockAssassination => self.has_identity(Identity::Contessa),
	    Action::Steal(..) => self.has_identity(Identity::Captain),
	}
    }

    fn has_identity(&self, identity: Identity) -> bool {
	self.get_hand().contains(&identity)
    }
}
