use crate::{Action, GameState, Identity, PlayerID};
use std::convert::TryInto;
use std::mem;
use anyhow::{anyhow, Result};

const MAX_CARDS: u8 = 2;

pub trait Player {
    /// A player must define how they choose an action. This will be called in the game loop, and given
    /// a snapshot of the game.
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
	    Err(_) => self.lose_life(state),
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

    fn get_other_active_players(&self, state: &GameState) -> Vec<PlayerID> {
	let mut other_players = vec!();
	for player_id in &state.active_players {
            if player_id != self.who_am_i() {
		other_players.push(player_id.clone());
            }
        }
	return other_players;
    }

    // Hm is enums with values an anti-pattern? fuq
    fn get_available_actions(&self, state: &GameState) -> Vec<Action> {
	let mut available_actions = vec!(Action::Income, Action::ForeignAid, Action::Tax, Action::Exchange);
	for target in &state.active_players {
	    if target != self.who_am_i() {
		available_actions.push(Action::Steal(target.clone()));
		if self.count_coins(state) >= 3 {
		    available_actions.push(Action::Assassinate(target.clone()));
		}
		if self.count_coins(state) >= 7 {
		    available_actions.push(Action::Coup(target.clone()));
		}
	    }
	}
	return available_actions;
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
	    Action::BlockStealCaptain => self.has_identity(Identity::Captain),
	    Action::BlockStealAmbassador => self.has_identity(Identity::Ambassador),
	}
    }

    fn has_identity(&self, identity: Identity) -> bool {
	self.get_hand().contains(&identity)
    }
}
