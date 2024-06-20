
#![warn(clippy::arithmetic_side_effects)]
#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[openbrush::implementation(Ownable)]
#[openbrush::contract]
pub mod vote_lunes{
    use openbrush::{
        modifiers,
        contracts::{
            ownable::{
                self,
                only_owner,
            },
            reentrancy_guard,
            traits::psp22::PSP22Error,
            reentrancy_guard::non_reentrant,
        },
        traits::{
            Storage, String
        },
    };
    use ink::{env::terminate_contract, storage::Mapping};

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct VoteLunes {
        #[storage_field]
        guard: reentrancy_guard::Data,
        #[storage_field]        
        ownable: ownable::Data,
        owner_project: Option<AccountId>,
        status: bool,
        price: Balance,
        qtd_vote: u64,
        vote: Mapping<AccountId, u64>,       
    }
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct InfoContract {
        pub qtd_vote: u64,
        pub price: Balance,
        pub owner_project: Option<AccountId>,
        pub status: bool,
    }

    impl VoteLunes {
        #[ink(constructor)]
        pub fn new(owner_project: Option<AccountId>, price:Balance) -> Self {
            let mut instance = Self::default();
            let caller = Self::env().caller();
            ownable::InternalImpl::_init_with_owner(&mut instance, caller);
            instance.owner_project = owner_project;
            instance.status = true;
            instance.price = price;
            instance.qtd_vote = 0;
            instance.vote = Mapping::default();
            instance
        }

        #[ink(message, payable)]
        #[modifiers(non_reentrant)]
        pub fn vote(&mut self) -> Result<(), PSP22Error> {
            // verificar na contrato psp22 se o usuario tem saldo
            // se tiver, votar
            if self.status == false {
                return Err(PSP22Error::Custom(String::from("Contract not active")));
            }
            let caller = self.env().caller();            
            let date_vote = self.vote.get(&caller);
            if date_vote.is_some() {
                return Err(PSP22Error::Custom(String::from("You have already voted")));
                
            }
            if Self::env().transferred_value() < self.price {
                return Err(PSP22Error::Custom(String::from("Insufficient price")));
            }
            self.qtd_vote += 1;
            self.vote.insert(&caller, &self.qtd_vote);
            
            Ok(())
        }
        
        #[ink(message)]
        pub fn get_qtd_votes(&mut self) -> Result<InfoContract, PSP22Error> {
            Ok(InfoContract {
                qtd_vote: self.qtd_vote,
                price: self.price,
                status: self.status,
                owner_project: self.owner_project
            })
        }

        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        pub fn stop(&mut self) -> Result<(), PSP22Error> {
            if self.status == false {
                return Err(PSP22Error::Custom(String::from("Contract not active")));
            }
            self.status = false;
            Ok(())
        }

        #[openbrush::modifiers(only_owner)]
        #[ink(message)]
        pub fn withdraw(&mut self) -> Result<(), PSP22Error> {
            if self.status {
                return Err(PSP22Error::Custom(String::from("Contract is active, please stop it first")));
            }
            let balance = Self::env().balance();
            let current_balance = balance
            .checked_sub(Self::env().minimum_balance())
            .unwrap_or_default();
            let owner = self.ownable.owner.get().unwrap().unwrap();
            Self::env()
                .transfer(owner, current_balance)
                .map_err(|_| PSP22Error::Custom(String::from("Error withdrawing balance, try again")))?;
            self.env().terminate_contract(self.env().caller())
        }
        pub fn vote_user(&self, to: AccountId) -> u64 {
            self.vote.get(&to).unwrap_or_default()
        }
    }
   
}
