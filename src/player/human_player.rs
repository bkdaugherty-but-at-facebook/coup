use crate::player::traits::Player;
use crate::{Action, GameState, Identity, PlayerID};
use crate::prompter::{Prompter, LocalPrompter};
use anyhow::{anyhow, Result};

pub struct HumanPlayer<P: Prompter> {
    // Not necessarily two?
    id: PlayerID,
    prompter: P,
    hand: Vec<Identity>,
}

impl<P: Prompter> HumanPlayer<P> {
    pub fn new(id: PlayerID, prompter: P) -> Self {
        let hand = Vec::new();
        HumanPlayer {
            id,
            prompter,
            hand,
        }
    }
}

impl<P: Prompter> Player for HumanPlayer<P> {
    fn choose_action(&self, state: &GameState) -> Action {
        let available_actions = self.get_available_actions(state);
        let action = self.prompter.prompt_player_for_action("What will you do?", available_actions, state);
        match action {
            Ok(action) => action,
            Err(e) => {
                println!("Hm. I didn't get that...");
                self.choose_action(state)
            }
        }
    }

    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
        let question = &format!(
            "Would you like to challenge {}'s {}?",
            state.get_player_name(player_id),
	    // TODO fix this! I hate that I genericized this but then it doesn't work.
	    // This should maybe be on game state?
            LocalPrompter::display_action(state, action.clone())
        );
        match self.prompter.prompt_player_yes_no(question, Some(state)) {
            Ok(x) => x,
            Err(e) => {
                // To do --> errors handled in prompter?
                println!("Hm. I didn't get that.");
                self.will_challenge(state, player_id, action)
            }
        }
    }
    fn will_block(
        &self,
        state: &GameState,
        player_id: &PlayerID,
        action: &Action,
    ) -> Option<Action> {
	let possible_actions = action.blockable(self.who_am_i());
        None
    }
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> Option<usize> {
        None
    }
    fn choose_card_to_lose(&self, state: &GameState) -> usize {
        0
    }
    fn choose_forced_coup(&self, state: &GameState) -> PlayerID {
        let other_players = self.get_other_active_players(state);
        return other_players[0].clone();
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
