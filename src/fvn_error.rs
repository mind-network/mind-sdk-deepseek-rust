impl crate::Fvn {
    pub async fn process_contract_call_error(
        &self,
        e: alloy::contract::Error,
    ) -> mind_sdk_util::MindError {
        match e {
            alloy::contract::Error::TransportError(rpc_error) => {
                return self.process_transport_error(rpc_error).await;
            }
            alloy::contract::Error::PendingTransactionError(pending_transaction_error) => {
                match pending_transaction_error {
                    alloy::providers::PendingTransactionError::TransportError(rpc_error) => {
                        return self.process_transport_error(rpc_error).await;
                    }
                    _e => {
                        return mind_sdk_util::MindError::MindContractOtherError(format!(
                            "Randgen: Unexpected contract error: {:#?}",
                            _e
                        ));
                    }
                }
            }
            _e => {
                return mind_sdk_util::MindError::MindContractOtherError(format!(
                    "Randgen: Unexpected contract error: {:#?}",
                    _e
                ));
            }
        }
    }

    pub async fn process_transport_error(
        &self,
        rpc_error: alloy::transports::RpcError<alloy::transports::TransportErrorKind>,
    ) -> mind_sdk_util::MindError {
        let rpc_error_response = rpc_error.as_error_resp();
        match rpc_error_response {
            Some(rpc_error) => {
                // Attempt to decode the transport error into contract ABI errors
                if let Some(abi_error) = rpc_error.as_decoded_error(true) {
                    match abi_error {
                        crate::fvn_contract::FvnContractAbi::FvnContractAbiErrors::GeneralError(
                            general_error,
                        ) => {
                            let contract_id = general_error.contractID;
                            let contract_id = mind_sdk_chain::error::parse_contract_id(contract_id);
                            let error_code = general_error.errorCode;
                            // 429: Duplicate submission detected for RandgenSubnet.
                            // 507: Insufficient gas funds:
                            // 304: Hot wallet already registered with voter wallet.
                            return mind_sdk_util::MindError::MindContractGeneralError(
                                mind_sdk_util::MindContractError {
                                    error_code,
                                    contract_name: contract_id,
                                    note: "from rsuban_randgen.fvn_error.process_transport_error function".to_string(),
                                    can_continue: false,
                                    from_wallet: self.onchain.account.address.to_string(),
                                }
                            );
                        }
                        _ => {
                            return mind_sdk_util::MindError::MindContractOtherError(
                                "Randgen: Undetected ABI contract".to_string(),
                            );
                        }
                    }
                } else {
                    return mind_sdk_util::MindError::MindContractOtherError(format!(
                        "Randgen: Unable to decode RPC error into ABI error: {:#?}",
                        rpc_error
                    ));
                }
            }
            None => {
                return mind_sdk_util::MindError::MindContractOtherError(format!(
                    "Randgen: RPC error without a response: {:#?}",
                    rpc_error
                ));
            }
        }
    }
}
