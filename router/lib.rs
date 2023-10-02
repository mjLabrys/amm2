#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
use ink_lang as ink;
const PRECISION: u128 = 1_000_000; //6 decimal precision

#[ink::contract]
mod router {
    use ink_storage::{traits::SpreadAllocate, Mapping};
    
    /// Specify ERC-20 error type.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Zero Liquidity
        ZeroLiquidity,
        /// Amount cannot be zero!
        ZeroAmount,
        /// Insufficient amount
        InsufficientAmount,
        /// Equivalent value of tokens not provided
        NonEquivalentValue,
        /// Asset value less than threshold for contribution!
        ThresholdNotReached,
        /// Share should be less than totalShare
        InvalidShare,
        /// Insufficient pool balance
        InsufficientLiquidity,
        /// Slippage tolerance exceeded
        SlippageExceeded,
    }
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Router {
        totalShares: Balance, // Stores the total amount of share issued for the pool
        totalToken1: Balance, // Stores the amount of Token1 locked in the pool
        totalToken2: Balance, // Stores the amount of Token2 locked in the pool
        shares: Mapping<AccountId, Balance>, // Stores the share holding of each provider
        token1Balance: Mapping<AccountId, Balance>, // Stores the token1 balance of each user
        token2Balance: Mapping<AccountId, Balance>, // Stores the token2 balance of each user
        fees: Balance,        // Percent of trading fees charged on trade
    }

    use ink_lang::utils::initialize_contract;
    #[ink(impl)]
    impl Router {
        #[ink(constructor)]
        pub fn new(_fees: Balance) -> Self {
            // Sets fees to zero if not in valid range
            initialize_contract(|contract: &mut Self| {
                contract.fees = if _fees >= 1000 { 0 } else { _fees };
            })
        }

        #[ink(message, payable)]
        pub fn deposit(&mut self){
            ink_env::debug_println!(
                "received payment: {}",
                self.env().transferred_value()
            );

            let caller = self.env().caller();
            let token1 = self.token1Balance.get(&caller).unwrap_or(0);

            let _transferred = self.env().transferred_value();
            self.token1Balance.insert(caller, &(token1 + _transferred));
        }

        #[ink(message)]
        pub fn getTest(&self) -> Balance{
            let caller = self.env().caller();
            self.token1Balance.get(&caller).unwrap_or(0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn should_initialize_with_default_setting(){
            let mut contract = Router::new(0);
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                accounts.alice, 100,
            );
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(accounts.alice);
            let bal = ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.alice);
            println!("Balance: {}", bal.unwrap_or(0));

            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(101);
            contract.deposit();


            let temp = contract.getTest();
            println!("TEST: {}", temp);
        }
    }
}
