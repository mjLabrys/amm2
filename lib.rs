#![cfg_attr(not(feature = "std"), no_std)]
#![allow(non_snake_case)]
use ink_lang as ink;

pub use self::factory::{
    Factory,
    FactoryRef,
};

#[ink::contract]
mod factory {
    use pair::PairRef;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::SpreadAllocate, 
        Mapping
    };
    #[derive(
        Debug,
        Copy,
        Clone,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
    )]
    #[cfg_attr(
        feature = "std",
        derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout)
    )]
    pub enum Error {
        IdenticalAddresses,
        PairExists,
        ZeroAddress,
        Unauthorized,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Factory {
        // (token0 address => token1 address) => pair address
        pairs: Mapping<(AccountId, AccountId), AccountId>,
        // pair address
        all_pairs: Vec<AccountId>,
        feeTo: AccountId,
        feeToSetter: AccountId,
    }

    #[ink(event)]
    pub struct PairCreated {
        #[ink(topic)]
        token0: Option<AccountId>,
        #[ink(topic)]
        token1: Option<AccountId>,
        pair: Option<AccountId>,
        current_pair_length: u64
    }

    use ink_lang::utils::initialize_contract;
    #[ink(impl)]
    impl Factory {
        #[ink(constructor)]
        pub fn new(_feeTo: AccountId, _feeToSetter: AccountId) -> Self {
            initialize_contract(|contract: &mut Self| {
                contract.feeTo = _feeTo;
                contract.feeToSetter = _feeToSetter;
            })
        }

        #[ink(message)]
        pub fn createPair(
            &mut self, 
            tokenA: AccountId,
            tokenB: AccountId,
            version: u32,
            pair_code_hash: Hash,
        ) -> Result<(), Error>{
            let ZERO_ADDRESS = AccountId::from([0x00; 32]); // zero address

            if tokenA == tokenB {
                return Err(Error::IdenticalAddresses)
            }
            if tokenA == ZERO_ADDRESS ||
                tokenB == ZERO_ADDRESS{
                return Err(Error::ZeroAddress)
            }     
            
            let token0; let token1;
            if tokenA < tokenB {
                token0 = tokenA;
                token1 = tokenB;
            }else{
                token0 = tokenB;
                token1 = tokenA;
            }

            let pair = self.pairs.get((&token0, &token1)).unwrap_or(ZERO_ADDRESS);
            if pair != ZERO_ADDRESS{
                return Err(Error::PairExists)
            }
            // Deploy token pair
            let total_balance = Self::env().balance();
            let salt = version.to_le_bytes();
            let pair_contract = PairRef::initialize(token0, token1)
                .endowment(total_balance/2)
                .code_hash(pair_code_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!(
                        "failed at instantiating the token pair contract: {:?}",
                        error
                    )
                });
            // 
            self.pairs.insert((token0, token1), &(pair_contract.getAccountId()));
            self.pairs.insert((token1, token0), &(pair_contract.getAccountId()));
            self.all_pairs.push(pair_contract.getAccountId());
            self.env().emit_event(PairCreated {
                token0: Some(token0),
                token1: Some(token1),
                pair: Some(pair_contract.getAccountId()),
                current_pair_length: self.all_pairs.len() as u64
            });
            Ok(())
        }

        #[ink(message)]
        pub fn setFeeTo(&mut self, _feeTo: AccountId) -> Result<(), Error>{
            let caller = self.env().caller();
            if caller != self.feeToSetter {
                return Err(Error::Unauthorized);
            }
            self.feeTo = _feeTo;
            Ok(())
        }

        #[ink(message)]
        pub fn setFeeToSetter(&mut self, _feeToSetter: AccountId) -> Result<(), Error>{
            let caller = self.env().caller();
            if caller != self.feeToSetter {
                return Err(Error::Unauthorized);
            }
            self.feeToSetter = _feeToSetter;
            Ok(())
        }

        #[ink(message)]
        pub fn feeTo(&self) -> AccountId{
            self.feeTo
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;
        // #[ink::test]
        // fn should_not_initialize_factory_with_identical_tokens(){
        //     let mut contract = Factory::new();

        //     let result = contract.createPair(
        //         AccountId::from([0x01; 32]),
        //         AccountId::from([0x01; 32])
        //     );
        //     assert_eq!(result, Err(Error::IdenticalAddresses));
        // }
        // #[ink::test]
        // fn should_not_initialize_factory_with_zero_address(){
        //     let mut contract = Factory::new();

        //     let mut result = contract.createPair(
        //         AccountId::from([0x00; 32]),
        //         AccountId::from([0x02; 32])
        //     );
        //     assert_eq!(result, Err(Error::ZeroAddress));

        //     result = contract.createPair(
        //         AccountId::from([0x01; 32]),
        //         AccountId::from([0x00; 32])
        //     );
        //     assert_eq!(result, Err(Error::ZeroAddress));
        // }
    }
}
