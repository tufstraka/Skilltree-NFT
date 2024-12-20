use candid::{CandidType, Principal};
use ic_cdk::{api, storage};
use ic_cdk::api::call::call;
use ic_cdk_macros::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cell::RefCell;
use log::{info};
use ic_ledger_types::{AccountIdentifier, Memo, Subaccount, Tokens, TransferArgs};

const ICP_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct SkillNFT {
    id: u64,
    title: String,
    description: String,
    creator: Principal,
    price: u64,
    unlock_duration: Option<u64>, // in nanoseconds
    metadata: HashMap<String, String>, // Additional details (e.g., level, requirements)
    owner: Principal,
    resale_price: Option<u64>,
    is_active: bool, 
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize, Default)]
struct SkillTreeStorage {
    nfts: HashMap<u64, SkillNFT>,
    next_id: u64,
    balances: HashMap<Principal, u64>,
    creator_royalties: HashMap<Principal, u64>,
}

thread_local! {
    static STATE: RefCell<SkillTreeStorage> = RefCell::new(SkillTreeStorage::default());
}

#[pre_upgrade]
fn pre_upgrade() {
    STATE.with(|state| {
        let state = state.borrow();
        storage::stable_save((state.clone(),)).expect("Failed to save state");
    });
}

#[post_upgrade]
fn post_upgrade() {
    let (saved_state,): (SkillTreeStorage,) = storage::stable_restore().expect("Failed to restore state");
    STATE.with(|state| {
        *state.borrow_mut() = saved_state;
    });
}

/// Mint a new SkillNFT.
#[update]
fn mint_skill_nft(
    title: String,
    description: String,
    price: u64,
    unlock_duration: Option<u64>,
    metadata: HashMap<String, String>
) -> Result<u64, String> {
    let creator = api::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let id = state.next_id;
        let nft = SkillNFT {
            id,
            title,
            description,
            creator,
            price,
            unlock_duration,
            metadata,
            owner: creator,
            resale_price: None,
            is_active: true,
        };
        state.nfts.insert(id, nft);
        state.next_id += 1;
        info!("SkillNFT minted with ID: {}", id);
        Ok(id)
    })
}

/// Purchase a SkillNFT.
#[update]
fn purchase_skill_nft(nft_id: u64) -> Result<(), String> {
    let buyer = api::caller();
    
    // First get NFT details
    let nft_details = STATE.with(|state| {
        let state = state.borrow();
        state.nfts.get(&nft_id).cloned()
    }).ok_or_else(|| "NFT not found".to_string())?;

    // Validate NFT status
    if !nft_details.is_active {
        return Err("NFT is not active".to_string());
    }
    if buyer == nft_details.owner {
        return Err("Cannot purchase your own NFT".to_string());
    }

    // Check buyer's balance
    let buyer_balance = STATE.with(|state| {
        let state = state.borrow();
        *state.balances.get(&buyer).unwrap_or(&0)
    });

    if buyer_balance < nft_details.price {
        return Err("Insufficient balance".to_string());
    }

    // Perform the purchase
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        // Update balances
        state.balances.insert(buyer, buyer_balance - nft_details.price);
        
        let creator_balance = *state.balances.get(&nft_details.creator).unwrap_or(&0);
        state.balances.insert(nft_details.creator, creator_balance + nft_details.price);

        // Update NFT ownership
        let mut nft = nft_details.clone();
        nft.owner = buyer;
        nft.resale_price = None;
        state.nfts.insert(nft_id, nft);

        // Update royalties
        let royalty = nft_details.price / 10; // 10% royalty
        let creator_royalty = *state.creator_royalties.get(&nft_details.creator).unwrap_or(&0);
        state.creator_royalties.insert(nft_details.creator, creator_royalty + royalty);

        info!("SkillNFT with ID: {} purchased by {:?}", nft_id, buyer);
        Ok(())
    })
}

/// Set a resale price for a purchased SkillNFT.
#[update]
fn set_resale_price(nft_id: u64, price: u64) -> Result<(), String> {
    let owner = api::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(nft) = state.nfts.get_mut(&nft_id) {
            if nft.owner != owner {
                return Err("Only the owner can set the resale price".to_string());
            }
            nft.resale_price = Some(price);
            info!("Resale price set for NFT ID: {}", nft_id);
            Ok(())
        } else {
            Err("NFT not found".to_string())
        }
    })
}

/// Retrieve NFT details.
#[query]
fn get_nft(nft_id: u64) -> Option<SkillNFT> {
    STATE.with(|state| state.borrow().nfts.get(&nft_id).cloned())
}

/// Get all NFTs for a specific user.
#[query]
fn get_user_nfts(user: Principal) -> Vec<SkillNFT> {
    STATE.with(|state| {
        state
            .borrow()
            .nfts
            .values()
            .filter(|nft| nft.owner == user)
            .cloned()
            .collect()
    })
}

/// Deactivate an NFT (e.g., if it violates policies).
#[update]
fn deactivate_nft(nft_id: u64) -> Result<(), String> {
    let caller = api::caller();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(nft) = state.nfts.get_mut(&nft_id) {
            if nft.creator != caller {
                return Err("Only the creator can deactivate the NFT".to_string());
            }
            nft.is_active = false;
            info!("NFT ID: {} has been deactivated", nft_id);
            Ok(())
        } else {
            Err("NFT not found".to_string())
        }
    })
}

/// Add balance to a user's account.
#[update]
async fn add_balance(amount: u64) -> Result<(), String> {
    let caller = api::caller();
    let canister_id = ic_cdk::id();

    // Convert amount to Tokens
    let tokens = Tokens::from_e8s(amount);

    let transfer_args = TransferArgs {
        memo: Memo(0),
        amount: tokens,
        fee: Tokens::from_e8s(10_000), // Standard ICP transaction fee
        from_subaccount: None,
        to: AccountIdentifier::new(&canister_id, &Subaccount([0; 32])),
        created_at_time: None,
    };

    // Call the ICP ledger canister to transfer tokens
    let transfer_result: Result<(u64,), _> = call(
        Principal::from_text(ICP_LEDGER_CANISTER_ID).unwrap(),
        "icrc1_transfer",
        (transfer_args,),
    ).await;

    match transfer_result {
        Ok((_block_index,)) => {
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                let balance = state.balances.entry(caller).or_insert(0);
                *balance += amount;
                info!("Added {} balance to {:?}", amount, caller);
                Ok(())
            })
        }
        Err(err) => Err(format!("Failed to call ledger canister: {:?}", err)),
    }
}

/// Get all active NFTs.
#[query]
fn get_active_nfts() -> Vec<SkillNFT> {
    STATE.with(|state| {
        state
            .borrow()
            .nfts
            .values()
            .filter(|nft| nft.is_active)
            .cloned()
            .collect()
    })
}

// Candid interface export
ic_cdk::export_candid!();
