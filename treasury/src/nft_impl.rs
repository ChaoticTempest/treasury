use crate::*;

pub const GAS_NFT_TRANSFER: Gas = 10_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NonFungibleTokens {
    pub account_id: AccountId,
    pub tokens: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NonFungibleToken {
  id: String,
  owner_id: String,
}

// TODO:
// * receive?
// * mint?
// * storage deposit???

#[near_bindgen]
impl Contract {
    /// Supported Fungible Tokens
    /// 
    /// ```bash
    /// near call treasury.testnet get_nft_list
    /// ```
    pub fn get_nft_list(&self) -> Vec<AccountId> {
        self.nft_holdings.keys_as_vector().to_vec()
    }

    /// Non-Fungible Tokens
    /// 
    /// ```bash
    /// near call treasury.testnet nft_holdings '{"from_index": 0, "limit": 10}'
    /// ```
    pub fn nft_holdings(
        &self,
        from_index: Option<U64>,
        limit: Option<U64>,
    ) -> Vec<NonFungibleTokens> {
        let mut result: Vec<NonFungibleTokens> = Vec::new();
        let mut start = 0;
        let mut end = 10;
        
        if let Some(from_index) = from_index {
            start = from_index.0;
        }
        if let Some(limit) = limit {
            end = u64::min(start + limit.0, self.nft_holdings.len());
        }

        // Return all tasks within range
        let keys = self.nft_holdings.keys_as_vector();
        for i in start..end {
            if let Some(account_id) = keys.get(i) {
                if let Some(tokens) = self.nft_holdings.get(&account_id) {
                    result.push(NonFungibleTokens { account_id, tokens });
                }
            }
        }

        result
    }

    /// Single Non-Fungible Token Balance
    /// NOTE: Unlike the FT standard, this account_id is the "fungible token account id"
    /// 
    /// ```bash
    /// near call treasury.testnet nft_tokens '{"account_id": "image.testnet"}'
    /// ```
    pub fn nft_tokens(
        &self,
        account_id: AccountId,
    ) -> Vec<NonFungibleToken> {
        let tokens = self.nft_holdings.get(&account_id);
        if tokens.is_none() { return Vec::new() }
        let token_ids = tokens.unwrap();

        token_ids.iter()
            .map(|id| NonFungibleToken {
                id: id.to_string(),
                owner_id: env::current_account_id()
            })
            .collect()
    }

    /// Transfer Non-Fungible Token
    /// NOTE: Assumes storage deposit has occurred for recipient
    /// 
    /// ```bash
    /// near call treasury.testnet nft_transfer '{"nft_account_id": "image.testnet", "to_account_id": "user.account.testnet", "to_token_id": "100000000000000000000000000000000"}' --accountId treasury.testnet
    /// ```
    pub fn nft_transfer(
        &mut self,
        nft_account_id: AccountId,
        to_token_id: String,
        to_account_id: AccountId,
    ) {
        assert_eq!(self.owner_id, env::predecessor_account_id(), "Must be owner");

        // Check if treasury holds the ft, and has enough balance
        let mut tokens = self.nft_holdings.get(&nft_account_id).expect("NFT Account not found");
        assert!(tokens.contains(&to_token_id), "NFT Token ID not found");

        // NOTE: Lame logic here, should be changed to callback
        let index = tokens.iter().position(|x| x == &to_token_id);
        if let Some(index) = index {
            tokens.remove(index);
            self.nft_holdings.insert(&nft_account_id, &tokens);
        }
        
        let p = env::promise_create(
            nft_account_id,
            b"nft_transfer",
            json!({
                "receiver_id": to_account_id,
                "token_id": to_token_id,
            }).to_string().as_bytes(),
            ONE_YOCTO,
            GAS_NFT_TRANSFER
        );

        env::promise_return(p);
    }

    // /// Could do anything useful to the approval-receiving contract, such as store the given
    // /// approval_id for use later when calling the NFT contract. Can also return whatever it wants,
    // /// maybe after further promise calls. This one simulates "return anything" behavior only.
    // /// Supports the following `msg` patterns:
    // /// * "return-now" - immediately return `"cool"`
    // /// * anything else - return the given `msg` after one more cross-contract call
    // fn nft_on_approve(
    //     &mut self,
    //     token_id: TokenId,
    //     owner_id: AccountId,
    //     approval_id: u64,
    //     msg: String,
    // ) -> PromiseOrValue<String> {
    //     // Verifying that we were called by non-fungible token contract that we expect.
    //     assert_eq!(
    //         &env::predecessor_account_id(),
    //         &self.non_fungible_token_account_id,
    //         "Only supports the one non-fungible token contract"
    //     );
    //     log!(
    //         "in nft_on_approve; sender_id={}, previous_owner_id={}, token_id={}, msg={}",
    //         &token_id,
    //         &owner_id,
    //         &approval_id,
    //         msg
    //     );
    //     match msg.as_str() {
    //         "return-now" => PromiseOrValue::Value("cool".to_string()),
    //         _ => {
    //             let prepaid_gas = env::prepaid_gas();
    //             let account_id = env::current_account_id();
    //             ext_self::ok_go(msg, &account_id, NO_DEPOSIT, prepaid_gas - GAS_FOR_NFT_ON_APPROVE)
    //                 .into()
    //         }
    //     }
    // }

    // /// Returns true if token should be returned to `sender_id`
    // /// Four supported `msg`s:
    // /// * "return-it-now" - immediately return `true`
    // /// * "keep-it-now" - immediately return `false`
    // /// * "return-it-later" - make cross-contract call which resolves with `true`
    // /// * "keep-it-later" - make cross-contract call which resolves with `false`
    // /// Otherwise panics, which should also return token to `sender_id`
    // fn nft_on_transfer(
    //     &mut self,
    //     sender_id: AccountId,
    //     previous_owner_id: AccountId,
    //     token_id: TokenId,
    //     msg: String,
    // ) -> PromiseOrValue<bool> {
    //     // Verifying that we were called by non-fungible token contract that we expect.
    //     assert_eq!(
    //         &env::predecessor_account_id(),
    //         &self.non_fungible_token_account_id,
    //         "Only supports the one non-fungible token contract"
    //     );
    //     log!(
    //         "in nft_on_transfer; sender_id={}, previous_owner_id={}, token_id={}, msg={}",
    //         &sender_id,
    //         &previous_owner_id,
    //         &token_id,
    //         msg
    //     );
    //     match msg.as_str() {
    //         "return-it-now" => PromiseOrValue::Value(true),
    //         "return-it-later" => {
    //             let prepaid_gas = env::prepaid_gas();
    //             let account_id = env::current_account_id();
    //             ext_self::ok_go(
    //                 true,
    //                 &account_id,
    //                 NO_DEPOSIT,
    //                 prepaid_gas - GAS_FOR_NFT_ON_TRANSFER,
    //             )
    //             .into()
    //         }
    //         "keep-it-now" => PromiseOrValue::Value(false),
    //         "keep-it-later" => {
    //             let prepaid_gas = env::prepaid_gas();
    //             let account_id = env::current_account_id();
    //             ext_self::ok_go(
    //                 false,
    //                 &account_id,
    //                 NO_DEPOSIT,
    //                 prepaid_gas - GAS_FOR_NFT_ON_TRANSFER,
    //             )
    //             .into()
    //         }
    //         _ => env::panic(b"unsupported msg"),
    //     }
    // }
}

