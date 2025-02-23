////////////////////////////////////////////////////////////////////////////////////////////////
pub use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[derive(Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
    #[sol(rpc=true, abi=true, alloy_sol_types=alloy::sol_types, docs=true)]
    FvnRoundContractAbi,
    "../resources/RandgenSubnetRound.json"
);

////////////////////////////////////////////////////////////////////////////////////////////////
pub(crate) type FvnRoundContractInstance =
    crate::fvn_round_contract::FvnRoundContractAbi::FvnRoundContractAbiInstance<
        alloy::transports::http::Http<reqwest::Client>,
        mind_sdk_chain::network::MindChainFillProvider,
    >;

////////////////////////////////////////////////////////////////////////////////////////////////
pub fn get_fvn_round_contract_instance(
    fhekeyregistry_contact_address: &alloy::primitives::Address,
    network_provider: &mind_sdk_chain::network::MindChainFillProvider,
) -> Result<crate::fvn_round_contract::FvnRoundContractInstance, mind_sdk_util::MindError> {
    let fvn_round_contract: crate::fvn_round_contract::FvnRoundContractInstance =
        crate::fvn_round_contract::FvnRoundContractAbi::new(
            fhekeyregistry_contact_address.clone(),
            network_provider.clone(),
        ); //
    return Ok(fvn_round_contract);
}
