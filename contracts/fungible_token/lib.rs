#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod fungible_token {
    
    use ink::storage::Mapping;

    #[ink::trait_definition]
    pub trait Erc20 {
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance;
        #[ink(message)]
        fn total_supply(&self) -> Balance;
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<Balance, Error>;
        #[ink(message)]
        fn get_owner(&self) -> AccountId;
        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<Balance, Error>;
    }
    

    #[ink(storage)]
    pub struct FungibleToken {
        owner: AccountId,
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
    }

    impl FungibleToken {
        /// Constructor that initializes the `FungibleToken`.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::new();
            let owner = Self::env().caller();
            balances.insert(owner, &total_supply);
            Self {
                owner,
                total_supply,
                balances,
            }
        }

        #[ink(message)]
        pub fn mint_to(&mut self, to: AccountId, value: Balance) {
            let caller = self.env().caller();
            assert_eq!(caller, self.owner);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, &(to_balance + value));
            // increase total supply
            self.total_supply += value;
        }
    }

    impl Erc20 for FungibleToken {
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        fn balance_of(&self, account: AccountId) -> Balance {
            self.balances.get(&account).unwrap_or_default()
        }

        #[ink(message)]
        fn get_owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<Balance, Error> {
            let from = self.env().caller();
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            let to_balance = self.balance_of(to);

            self.balances.insert(from, &(from_balance - value));
            self.balances.insert(to, &(to_balance + value));

            Ok(self.balance_of(from))
        }

        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<Balance, Error> {
            let caller = self.env().caller();
            // TO-DO: need to check if the caller is allowed to transfer from `from`

            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            let to_balance = self.balance_of(to);

            self.balances.insert(from, &(from_balance - value));
            self.balances.insert(to, &(to_balance + value));

            Ok(self.balance_of(from))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn total_supply_works() {
            let mytoken = FungibleToken::new(100);
            assert_eq!(mytoken.total_supply(), 100);
        }

        #[ink::test]
        fn balance_of_works() {
            let mytoken = FungibleToken::new(100);
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            assert_eq!(mytoken.balance_of(accounts.alice), 100);
            assert_eq!(mytoken.balance_of(accounts.bob), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let total_supply = 100;
            let quantity_to_bob = 10;
            let mut mytoken = FungibleToken::new(total_supply);
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            assert_eq!(mytoken.balance_of(accounts.bob), 0);
            assert_eq!(
                mytoken.transfer(accounts.bob, quantity_to_bob),
                Ok(total_supply - quantity_to_bob)
            );
            assert_eq!(mytoken.balance_of(accounts.bob), quantity_to_bob);
        }

        #[ink::test]
        fn mint_to_works() {
            let total_supply = 100;
            let quantity_to_bob = 10;
            let mut mytoken = FungibleToken::new(total_supply);
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

            assert_eq!(mytoken.balance_of(accounts.bob), 0);
            mytoken.mint_to(accounts.bob, quantity_to_bob);
            assert_eq!(mytoken.balance_of(accounts.bob), quantity_to_bob);
            assert_eq!(mytoken.total_supply(), total_supply + quantity_to_bob);
        }
    }
}
