// This is an example contract.
// You can build it with scarb, and then you will be able to analyze it
// using the sierra-decompiler binary 

#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn strings_detector(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn felt_overflow(ref self: TContractState, amount: felt252);
}

#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252, 
    }

    #[abi(embed_v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Trigger the string detector
        fn strings_detector(ref self: ContractState, amount: felt252) {
            assert(amount != 0, 'This is a string');
            self.balance.write(self.balance.read() + amount);
        }

        // Trigger the felt overflow detector 
        fn felt_overflow(ref self: ContractState, amount: felt252) {
            let _overflow = 1000 + amount;
        }

        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
        
    }
}
