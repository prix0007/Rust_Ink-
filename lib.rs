#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    #[cfg(not(feature = "ink-as-dependency"))]
    #[ink(storage)]
    pub struct Erc20 {
        /// The total supply.
        total_supply: Balance,
        /// The balance of each user.
        balances: ink_storage::collections::HashMap<AccountId, Balance>,
        allowances: ink_storage::collections::HashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            // ACTION: `set` the total supply to `initial_supply`
            // ACTION: `insert` the `initial_supply` as the `caller` balance
            let caller = Self::env().caller();
            let mut balances = ink_storage::collections::HashMap::new();
            balances.insert(caller, initial_supply);
            let allowances = ink_storage::collections::HashMap::new();

            Self::env()
                .emit_event(
                    Transfer {
                        from: None,
                        to: Some(caller),
                        value: initial_supply,
                    });


            Self {
                total_supply: initial_supply,
                balances,
                allowances
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            // ACTION: Return the total supply
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            // ACTION: Return the balance of `owner`
            //   HINT: Use `balance_of_or_zero` to get the `owner` balance
            self.balance_of_or_zero(&owner)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            // ACTION: Get the `self.env().caller()` and store it as the `owner`
            // ACTION: Insert the new allowance into the `allowances` HashMap
            //   HINT: The key tuple is `(owner, spender)`
            // ACTION: `emit` the `Approval` event you created using these values
            // ACTION: Return true if everything was successful
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);

            // Notify offchain users of the approval and report success.
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            true

        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            // ACTION: Create a getter for the `allowances` HashMap
            //   HINT: Take a look at the getters above if you forget the details
            // ACTION: Return the `allowance` value
            self.allowance_of_or_zero(&owner, &spender)
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            // Ensure that a sufficient allowance exists.
            let caller = self.env().caller();
            let allowance = self.allowance_of_or_zero(&from, &caller);
            if allowance < value {
                return false;
            }

            let transfer_result = self.transfer_from_to(from, to, value);
            // Check `transfer_result` because `from` account may not have enough balance
            //   and return false.
            if !transfer_result {
                return false
            }

            // Decrease the value of the allowance and transfer the tokens.
            self.allowances.insert((from, caller), allowance - value);
            true
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            // ACTION: Call the `transfer_from_to` with `from` as `self.env().caller()`
           self.transfer_from_to(self.env().caller(), to, value)
        }

        
        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> bool {
            // ACTION: Get the balance for `from` and `to`
            //   HINT: Use the `balance_of_or_zero` function to do this
            // ACTION: If `from_balance` is less than `value`, return `false`
            // ACTION: Insert new values for `from` and `to`
            //         * from_balance - value
            //         * to_balance + value
            // ACTION: Return `true`
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < value {
                return false
            } 
            // Update the sender's balance.
            self.balances.insert(from, from_balance - value);

            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(to, to_balance + value);

            self.env()
                .emit_event(
                    Transfer {
                        from: Some(from),
                        to: Some(to),
                        value: value,
                    });


            true


        }
        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn allowance_of_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            // ACTION: `get` the `allowances` of `(owner, spender)` and `unwrap_or` return `0`.
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
        }

      
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let contract = Erc20::new(777);
            assert_eq!(contract.total_supply(), 777);
        }

        #[ink::test]
        fn balance_works() {
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert!(contract.transfer(AccountId::from([0x0; 32]), 10));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
            assert!(!contract.transfer(AccountId::from([0x0; 32]), 100));
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract.approve(AccountId::from([0x1; 32]), 20);
            contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
        }

    }
}
