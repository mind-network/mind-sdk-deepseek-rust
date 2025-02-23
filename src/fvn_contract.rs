////////////////////////////////////////////////////////////////////////////////////////////////
pub use alloy::sol;
sol!(
    #[allow(missing_docs)]
    #[derive(Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
    #[sol(rpc=true, abi=true, alloy_sol_types=alloy::sol_types, docs=true)]
    FvnContractAbi,
    "../resources/RandgenSubnet.json"
);

////////////////////////////////////////////////////////////////////////////////////////////////
pub type FvnContractInstance = crate::fvn_contract::FvnContractAbi::FvnContractAbiInstance<
    alloy::transports::http::Http<reqwest::Client>,
    mind_sdk_chain::network::MindChainFillProvider,
>;

pub type FvnContractInstanceSubmitRandomCtCall<'a> = alloy::contract::CallBuilder<
    alloy::transports::http::Http<reqwest::Client>,
    &'a mind_sdk_chain::network::MindChainFillProvider,
    std::marker::PhantomData<FvnContractAbi::submitRandomCtCall>,
>;

////////////////////////////////////////////////////////////////////////////////////////////////
pub fn get_fvn_contract_instance(
    subnet_contact_address: &alloy::primitives::Address,
    network_provider: &mind_sdk_chain::network::MindChainFillProvider,
) -> Result<crate::fvn_contract::FvnContractInstance, mind_sdk_util::MindError> {
    let randgen_contract: crate::fvn_contract::FvnContractInstance =
        crate::fvn_contract::FvnContractAbi::new(
            subnet_contact_address.clone(),
            network_provider.clone(),
        );
    return Ok(randgen_contract);
}
