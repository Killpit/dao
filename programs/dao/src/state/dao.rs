use anchor_lang::prelude::*;

use crate::constants::*;

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum Time {
    FiveSeconds,
    TwentyFourHours,
    FourtyEightHours,
    OneWeek
}

impl Time {
    pub fn value(&self) -> i64 {
        match *self {
            Time::FiveSeconds => 5, // for testing purposes only
            Time::TwentyFourHours => ONE_DAY_IN_SECONDS,
            Time::FourtyEightHours => TWO_DAY_IN_SECONDS,
            Time::OneWeek => ONE_WEEK_IN_SECONDS,
        }
    }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum Choice {
    Approve,
    Reject,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum Status {
    Approved,
    Rejected,
    Voting
}

#[account]
pub struct DAO {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub time: i64,
    pub threshold: u8,
    pub min_poll_tokens: u64,
    pub approved: u64,
    pub rejected: u64,
    pub created_at: i64,
    pub dao_bump: u8,
    pub vault_bump: u8,
    pub name: String,
    pub polls: Vec<Poll>,
    pub users: Vec<User>,
}

impl DAO {
    pub const LEN: usize = DISCRIMINATOR_LENGTH
        + PUBLIC_KEY_LENGTH * 2 // creator, mint 
        + 1 + 8
        + 1 // threshold 51 => 100
        + 8 * 3 // approved, rejected, min_poll_tokens 
        + TIMESTAMP_LENGTH * 2 // time, created_at
        + BUMP_LENGTH // bump
        + VECTOR_LENGTH_PREFIX * 2
        + STRING_LENGTH_PREFIX
        + MAX_DAO_NAME_LENGTH;

    pub fn total_deposits(&self) -> usize {
        self.users.iter().map(|user| user.deposits.len()).sum()
    }
    pub fn total_polls(&self) -> usize {
        self.polls.len()
    }

    pub fn total_deposit_amount(&self) -> u64 {
        self.users.iter().map(|user| {
            user.deposits.iter().map(|deposit| deposit.amount).sum::<u64>()
        }).sum()
    }

    pub fn total_votes(&self) -> usize {
        self.polls.iter().map(|poll| poll.votes.len()).sum()
    }
    
    pub fn reward_points(&mut self, poll_id: usize) {
        if let Some(poll) = self.polls.get(poll_id) {
            for vote in &poll.votes {
                if let Some(user) = self.users.iter_mut().find(|user| user.user == vote.user) {
                    user.points += vote.voting_power;
                }
            }
        }
    }
}




#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub struct Poll {
    pub creator: Pubkey,
    pub created_at: i64,
    pub executed: bool,
    pub status: Status,
    pub title: String,
    pub content: String,
    pub votes: Vec<Vote>,
}

impl Poll {
    pub const LEN: usize = DISCRIMINATOR_LENGTH
        + PUBLIC_KEY_LENGTH // creator 
        + TIMESTAMP_LENGTH // created_at
        + BOOL_LENGTH 
        + STRING_LENGTH_PREFIX * 2
        + MAX_TITLE_LENGTH
        + MAX_CONTENT_LENGTH
        + VECTOR_LENGTH_PREFIX; // bump

    pub fn is_approved(&self, dao: &DAO) -> bool {
        let mut approve_power = 0u64;
        let mut total_power = 0u64;

        for vote in &self.votes {
            total_power += vote.voting_power;
            if vote.choice == Choice::Approve {
                approve_power += vote.voting_power;
            }
        }

        let approval_percentage = (approve_power as f64 / total_power as f64) * 100.0;
        approval_percentage >= dao.threshold as f64
    }}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub struct Vote {
    pub user: Pubkey,
    pub voting_power: u64,
    pub choice: Choice,
    pub created_at: i64,
}

impl Vote {
    pub const LEN: usize = PUBLIC_KEY_LENGTH 
        + 8 // voting_power 
        + 1 // enum
        + TIMESTAMP_LENGTH;
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub struct User {
    pub user: Pubkey,
    pub voting_power: u64,
    pub points: u64,
    pub created_at: i64,
    pub deposits: Vec<Deposit>,
}

impl User {
    pub const LEN: usize = PUBLIC_KEY_LENGTH
        + 8
        + TIMESTAMP_LENGTH
        + VECTOR_LENGTH_PREFIX;

    pub fn total_user_deposit_amount(&self) -> u64 {
        self.deposits.iter().map(|deposit| {
            if !deposit.deactivating {deposit.amount} else {0u64}
        }).sum()
    }
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub struct Deposit {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub deactivating: bool,
    pub deactivation_start: Option<i64>,
    pub created_at: i64,
}

impl Deposit {
    pub const LEN: usize = PUBLIC_KEY_LENGTH * 2 
        + 8 // amount
        + BOOL_LENGTH // bool
        + 1 // option
        + TIMESTAMP_LENGTH * 2;
}
