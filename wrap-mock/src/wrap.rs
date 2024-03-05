#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait WrapMock<ContractReader> {
    #[init]
    fn init(&self, wegld_id: TokenIdentifier) {
        self.wrapped_egld_token_id().set(wegld_id);
    }

    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(&self) -> EsdtTokenPayment<Self::Api> {
        let payment_amount = self.call_value().egld_value().clone_value();
        require!(payment_amount > 0u32, "Payment must be more than 0");

        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        let caller = self.blockchain().get_caller();
        self.send()
            .direct_esdt(&caller, &wrapped_egld_token_id, 0, &payment_amount);

        EsdtTokenPayment::new(wrapped_egld_token_id, 0, payment_amount)
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(&self) {
        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();
        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        require!(payment_token == wrapped_egld_token_id, "Wrong esdt token");
        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");

        let caller = self.blockchain().get_caller();
        self.send().direct_egld(&caller, &payment_amount);
    }

    #[view(getWrappedEgldTokenId)]
    #[storage_mapper("wrappedEgldTokenId")]
    fn wrapped_egld_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
