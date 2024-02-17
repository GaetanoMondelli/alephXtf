#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod escrow {

    use blake2_rfc::blake2b::Blake2b;
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        DefaultEnvironment,
    };
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum EscrowError {
        InsufficientBalance,
        UnsupportedToken,
        TransferFailed,
    }

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        token: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct WithdrawAll {}


    #[ink(event)]
    pub struct Withdrawal {
        #[ink(topic)]
        token: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink::trait_definition]
    pub trait Erc20 {
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance;
        #[ink(message)]
        fn total_supply(&self) -> Balance;
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<Balance, EscrowError>;
        #[ink(message)]
        fn get_owner(&self) -> AccountId;
    }

    #[ink(storage)]
    pub struct Escrow {
        // list of supported tokens
        tokens: Vec<AccountId>,
        // mapping from token to balance
        balances: Mapping<AccountId, Balance>,
        // admin of the escrow
        admin: AccountId,
    }

    impl Escrow {
        fn calculate_selector(function_name: &str) -> [u8; 4] {
            let mut hasher = Blake2b::new(32);
            hasher.update(function_name.as_bytes());
            let result = hasher.finalize();
            let mut selector = [0u8; 4];
            selector.copy_from_slice(&result.as_bytes()[0..4]);
            selector
        }

        #[ink(constructor)]
        pub fn new(supported_tokens: Vec<AccountId>) -> Self {
            Self {
                tokens: supported_tokens,
                balances: Mapping::new(),
                admin: Self::env().caller(),
            }
        }

        #[ink(message)]
        pub fn get_tokens(&self) -> Vec<AccountId> {
            self.tokens.clone()
        }

        #[ink(message)]
        pub fn get_balance(&self, token: AccountId) -> Balance {
            self.balances.get(&token).unwrap_or_default()
        }

        #[ink(message)]
        pub fn deposit(&mut self, token: AccountId, amount: Balance) -> Result<(), EscrowError> {
            let caller = self.env().caller();

            // Check if the token is supported
            if !self.tokens.contains(&token) {
                return Err(EscrowError::UnsupportedToken);
            }

            // Get the selector for the transfer function
            let transfer_selector = Escrow::calculate_selector("transfer");
            let transfer_selector = Selector::new(transfer_selector);

            build_call::<DefaultEnvironment>()
                .call(token)
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(transfer_selector)
                        .push_arg(caller)
                        .push_arg(amount),
                )
                .returns::<()>()
                .invoke();

            // Emit the deposit event
            self.env().emit_event(Deposit { token, amount });

            // Update the balances
            let balance = self.get_balance(token);
            self.balances.insert(token, &(balance + amount));
            Ok(())
        }

        #[ink(message)]
        pub fn withdraw(&mut self, token: AccountId, amount: Balance) -> Result<(), EscrowError> {
            let caller = self.env().caller();
            // only the admin can withdraw
            if caller != self.admin {
                return Err(EscrowError::TransferFailed);
            }

            // Check if the token is supported
            if !self.tokens.contains(&token) {
                return Err(EscrowError::UnsupportedToken);
            }

            // Check if the balance is sufficient
            let balance = self.get_balance(token);
            if balance < amount {
                return Err(EscrowError::InsufficientBalance);
            }

            // Get the selector for the transfer function
            let transfer_selector = Escrow::calculate_selector("transfer");
            let transfer_selector = Selector::new(transfer_selector);

            build_call::<DefaultEnvironment>()
                .call(token)
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(transfer_selector)
                        .push_arg(caller)
                        .push_arg(amount),
                )
                .returns::<()>()
                .invoke();

            // Update the balances
            self.balances.insert(token, &(balance - amount));
            Ok(())
        }

        #[ink(message)]
        pub fn withdraw_all(&mut self) -> Result<(), EscrowError> {
            let caller = self.env().caller();
            // only the admin can withdraw
            if caller != self.admin {
                return Err(EscrowError::TransferFailed);
            }

            for token in self.tokens.iter() {
                let balance = self.get_balance(*token);
                if balance > 0 {
                    // Get the selector for the transfer function
                    let transfer_selector = Escrow::calculate_selector("transfer");
                    let transfer_selector = Selector::new(transfer_selector);

                    build_call::<DefaultEnvironment>()
                        .call(*token)
                        .gas_limit(0)
                        .transferred_value(0)
                        .exec_input(
                            ExecutionInput::new(transfer_selector)
                                .push_arg(caller)
                                .push_arg(balance),
                        )
                        .returns::<()>()
                        .invoke();

                    // Update the balances
                    self.balances.insert(*token, &0);
                }
            }
            self.env().emit_event(WithdrawAll {});
            Ok(())
        }

        #[ink(message)]
        pub fn get_admin(&self) -> AccountId {
            self.admin
        }

        #[ink(message)]
        pub fn set_admin(&mut self, new_admin: AccountId) {
            let caller = self.env().caller();
            if caller == self.admin {
                self.admin = new_admin;
            }
        }
    }
}
