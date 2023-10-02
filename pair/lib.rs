#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]

const PRECISION: u128 = 1_000_000; //6 decimal precision
const MINIMUM_LIQUIDITY: u128 = 1_000;
pub use self::pair::{
    Pair,
    PairRef,
};
use ink_lang as ink;

#[ink::contract]
mod pair {
    // Contract constants
    const MAX_BAL: Balance = u64::MAX as Balance;

    // Cross contracts
    use erc20::Erc20Ref;
    // use factory::FactoryRef;

    use ink_storage::{traits::SpreadAllocate, Mapping};
    use std::cmp;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout)
    )]
    pub enum Error {
        AlreadyInitialized,
        BalanceOverflow,
        InsufficientInputAmount,
        InsufficientLiquidity,
        InsufficientLiquidityBurned,
        InsufficientLiquidityMinted,
        InsufficientOutputAmount,
        InvalidK,
        TransferFailed,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Pair {
        token0: AccountId,
        token1: AccountId,
        reserve0: Balance,
        reserve1: Balance,
        blockTimestampLast: Timestamp,
        price0CumulativeLast: Balance,
        price1CumulativeLast: Balance,
        isEntered: bool,

        factory: AccountId,

        // Erc20
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
    }

    // #[ink(event)]
    // pub struct Transfer {
    //     #[ink(topic)]
    //     from: Option<AccountId>,
    //     #[ink(topic)]
    //     to: Option<AccountId>,
    //     value: Balance,
    // }

    // #[ink(event)]
    // pub struct Sync {
    //     reserve0: Balance,
    //     reserve1: Balance,
    // }

    // #[ink(event)]
    // pub struct Mint {
    //     #[ink(topic)]
    //     sender: Option<AccountId>,
    //     amount0: Balance,
    //     amount1: Balance,
    // }

    use ink_lang::utils::initialize_contract;
    impl Pair {
        #[ink(constructor)]
        pub fn initialize(_token0: AccountId, _token1: AccountId) -> Self {
            initialize_contract(|contract: &mut Self| {
                let caller = Self::env().caller();
                contract.factory = caller;
                contract.token0 = _token0; 
                contract.token1 = _token1;
            })
        }
        
        #[ink(message)]
        pub fn mint(&mut self, to: AccountId) -> Result<Balance, Error>{
            let (reserve0_, reserve1_, timestamp_) = self.getReserves();
            let token0: Erc20Ref = ink_env::call::FromAccountId::from_account_id(self.token0);
            let token1: Erc20Ref = ink_env::call::FromAccountId::from_account_id(self.token1);

            let balance0 = token0.balance_of(self.getAccountId());
            let balance1 = token1.balance_of(self.getAccountId());

            let amount0 = balance0 - reserve0_;
            let amount1 = balance1 - reserve1_;

            let liquidity;
            
            if self.total_supply == 0 {
                liquidity = (Self::sqrt(amount0 * amount1) - super::MINIMUM_LIQUIDITY) * super::PRECISION;
                self._mint(AccountId::from([0x00; 32]), super::MINIMUM_LIQUIDITY * super::PRECISION);
            }else{
                liquidity = cmp::min(
                    (amount0 * self.total_supply) / reserve0_,
                    (amount1 * self.total_supply) / reserve1_
                );
            }
            if liquidity <= 0 {
                return Err(Error::InsufficientLiquidityMinted)
            }

            self._mint(to, liquidity);
            self._update(balance0, balance1, reserve0_, reserve1_);

            // self.env().emit_event(Mint {
            //     sender: Some(to),
            //     amount0: amount0,
            //     amount1: amount1
            // });

            Ok(liquidity as Balance)
        }

        // #[ink(message)]
        // pub fn burn(&mut self, to: AccountId) -> Result<(Balance, Balance), Error> {

        // }

        #[ink(message)]
        pub fn getReserves(&self) -> (Balance, Balance, Timestamp) {
            (
                self.reserve0,
                self.reserve1,
                self.blockTimestampLast
            )
        }

        #[inline]
        fn _update(
            &mut self, 
            balance0: Balance, 
            balance1: Balance, 
            reserve0_ : Balance, 
            reserve1_: Balance
        ) -> Result<(), Error> {
            if balance0 > MAX_BAL || balance1 > MAX_BAL {
                return Err(Error::BalanceOverflow);
            }

            let timeElapsed = self.env().block_timestamp() - self.blockTimestampLast;
            if timeElapsed > 0 && reserve0_ > 0 && reserve1_ > 0 {
                self.price0CumulativeLast += reserve1_ * super::PRECISION / reserve0_;
                self.price1CumulativeLast += reserve0_ * super::PRECISION / reserve1_;
            }
            self.reserve0 = balance0;
            self.reserve1 = balance1;
            self.blockTimestampLast = self.env().block_timestamp();

            // self.env().emit_event(Sync {
            //     reserve0: self.reserve0,
            //     reserve1: self.reserve1
            // });
            Ok(())
        }

        fn _mintFee(
            &self,
            _reserve0: Balance,
            _reserve1: Balance
        ) -> bool{
            // let token0: Erc20Ref = ink_env::call::FromAccountId::from_account_id(self.token0);
            let factoryContract: FactoryRef = ink_env::call::FromAccountId::from_account_id(self.factory);
            

            true
        }

        // ERC20 Share token
        fn _mint(&mut self, to: AccountId, amount: Balance) -> Result<(), Error>{
            self.total_supply += amount;

            let to_balance = self.balance_of_impl(&to);
            self.balances.insert(&to, &(to_balance + amount));
            Ok(())
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_impl(&owner)
        }

        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        // Helper functions
        #[ink(message)]
        pub fn getAccountId(&self) -> AccountId {
            self.env().account_id()
        }

        #[inline]
        fn sqrt(_qty: Balance) -> Balance {
            _qty * _qty
        }    
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        #[ink::test]
        fn should_initialize_properly(){
            let mut contract = Pair::initialize(
                AccountId::from([0x01; 32]),
                AccountId::from([0x02; 32])
            );
            // let test =  contract.test();
            // assert_eq!(contract.token0, AccountId::from([0x01; 32]));
            // assert_eq!(contract.token1, AccountId::from([0x02; 32]));
            // contract.mint(AccountId::from([0x03; 32]));
            // println!("jknsjf: {}", contract.MAX_BAL());
        }
    }
}
