impl crate::Fvn {
    // for randgen

    pub async fn check_round(&self) -> Result<(), mind_sdk_util::MindError> {
        let current_round = self.fvn_round_contract.currentRoundNumber().call().await?;
        let current_round_number = current_round._0.to_string().parse::<u64>()?;

        let used_round = self.fvn_round_contract.usedRoundNumber().call().await?;
        let used_round_number = used_round._0.to_string().parse::<u64>()?;

        println!("current round: {:#?}", current_round_number);
        println!("used round: {:#?}", used_round_number);

        let current_round_details = self
            .fvn_round_contract
            .rounds(alloy::primitives::Uint::from(current_round_number))
            .call()
            .await?;
        println!("current round details: {:#?}", current_round_details);

        let current_round_details = self
            .fvn_round_contract
            .rounds(alloy::primitives::Uint::from(current_round_number - 1))
            .call()
            .await
            .unwrap();
        println!("current-1 round details: {:#?}", current_round_details);

        let current_round_details = self
            .fvn_round_contract
            .rounds(alloy::primitives::Uint::from(current_round_number - 2))
            .call()
            .await?;
        println!("current-2 round details: {:#?}", current_round_details);

        let used_round_details = self
            .fvn_round_contract
            .rounds(alloy::primitives::Uint::from(used_round_number))
            .call()
            .await?;
        println!("used round details: {:#?}", used_round_details);
        Ok(())
    }

    pub async fn check_if_voter_ready(&self) -> Result<bool, mind_sdk_util::MindError> {
        let hot_wallet_address = self.onchain.account.address;
        let voter = self
            .fvn_contract
            .isVoterReady(hot_wallet_address.clone())
            .call()
            .await?;
        log::debug!("{:#?} {:#?}", &voter, &self.onchain.account.address);
        if voter._0.to_string() == "404".to_string() {
            log::error!(
                "hot_wallet: {:?} is not registered, please register before voting ...",
                hot_wallet_address
            );
            return Ok(false);
        } else {
            if voter._0 != alloy::primitives::U256::ZERO {
                // not registered on chain
                return Ok(false);
            } else {
                let hot_wallet_address_eth_balance = self.onchain.get_eth_balance().await?;

                if hot_wallet_address_eth_balance
                    >= alloy::primitives::U256::from(self.basic_config.min_gas)
                {
                    return Ok(true);
                } else {
                    // hot_wallet has no enought gas
                    log::error!(
                        "hot_wallet has no enough gas ! hot wallet: {:?}, has: {:?}, require:{:?}",
                        &self.onchain.account.address,
                        hot_wallet_address_eth_balance,
                        self.basic_config.min_gas
                    );
                    return Ok(false); // TODO to change back
                }
            }
        }
    }

    pub async fn get_voter_wallet(
        &self,
    ) -> Result<Option<alloy::primitives::Address>, mind_sdk_util::MindError> {
        let registered_voter_address = self
            .fvn_contract
            .hotWalletToVoter(self.onchain.account.address)
            .call()
            .await?;
        let registered_voter_address = registered_voter_address.voter;
        log::debug!("{:#?}", &registered_voter_address);
        if registered_voter_address.clone().to_string()
            == "0x0000000000000000000000000000000000000000"
        {
            return Ok(None);
        } else {
            return Ok(Some(registered_voter_address));
        }
    }

    pub async fn check_hot_voter_registration(
        &self,
        hot_wallet_address: &alloy::primitives::Address,
        voter_wallet_address: &alloy::primitives::Address,
    ) -> Result<bool, mind_sdk_util::MindError> {
        let registered_voter_address = self
            .fvn_contract
            .hotWalletToVoter(hot_wallet_address.clone())
            .call()
            .await?;
        let registered_voter_address = registered_voter_address.voter;
        if registered_voter_address == voter_wallet_address.clone() {
            return Ok(true);
        } else {
            if registered_voter_address.clone().to_string()
                == "0x0000000000000000000000000000000000000000"
            {
                log::debug!(
                    "hot_wallet_is_new_and_has_no_hot_wallet_registered !: {:?}",
                    hot_wallet_address
                );
                return Ok(false);
            } else {
                log::error!(
                "hot_wallet_is_registered_with_other_hot_wallet_error !: hot: {:?}, other voter_wallet: {:?}, and not match to expected voter wallet: {:?}",
                hot_wallet_address, registered_voter_address, voter_wallet_address
                );
                return Ok(false);
            }
        }
    }

    pub async fn hot_wallet_send_voter_registration(
        &self,
        voter_wallet_address: alloy::primitives::Address,
    ) -> Result<alloy::rpc::types::TransactionReceipt, mind_sdk_util::MindError> {
        let registration_status = self
            .check_hot_voter_registration(&self.onchain.account.address, &voter_wallet_address)
            .await?;
        match registration_status {
            true => {
                log::debug!("voter and hot is already registered, will register again");
                return Err(mind_sdk_util::MindError::VoterAlreadyRegistered(format!(
                    "duplicated voter registration: hot: {:#?}, voter: {:#?}",
                    &self.onchain.account.address, &voter_wallet_address
                )));
            }
            false => {
                log::debug!(
                    "voter and hot is not registered or registered with others, will register here"
                );
            }
        }

        // Step 1: Set up the transaction with the gas amount and wallet address
        let contract_call_builder = self
            .fvn_contract
            .registerVoter(voter_wallet_address.clone());

        // Step 2: Run the simulation
        let estimated_gas = contract_call_builder.estimate_gas().await?;

        let new_gas_amount = estimated_gas;
        let tx = contract_call_builder
            .clone()
            .gas(new_gas_amount)
            .from(self.onchain.account.address);

        match tx.call().await {
            Ok(simulation_result) => {
                log::debug!("Simulation successful: {:?}", simulation_result);
                // Only proceed with sending the transaction if the simulation is successful
                let send_result = tx.send().await?;

                // Step 3: Wait for the transaction receipt
                let tx_receipt: alloy::rpc::types::TransactionReceipt =
                    send_result.get_receipt().await?;

                log::info!(
                    "Confirmed TX: status: {:?}, hash: {:?}, gas: {:?}, block: {:?}",
                    &tx_receipt.status(),
                    &tx_receipt.transaction_hash,
                    &tx_receipt.gas_used,
                    &tx_receipt.block_number.unwrap_or(0)
                );
                Ok(tx_receipt)
            }
            Err(e) => {
                let _r = mind_sdk_util::jsonlog(
                    "error",
                    480,
                    "call simulation error",
                    serde_json::json!({"caller": self.onchain.account.address, "error": format!("{:#?}", &e)}),
                )?;
                log::error!("Simulation failed: {:?}", e);
                Err(mind_sdk_util::MindError::ContractCallSimulationError(e))
            }
        }
    }

    pub async fn register_voter_wallet(
        &self,
        voter_wallet_address: alloy::primitives::Address,
    ) -> Result<(), mind_sdk_util::MindError> {
        let tx_receipt = self
            .hot_wallet_send_voter_registration(voter_wallet_address)
            .await?;
        log::debug!("registration is done: {:#?}", tx_receipt);
        let voter_ready = self.check_if_voter_ready().await?;
        if voter_ready {
            log::info!(
                "Voter is correctly set: hot: {:?}, voter: {:?}",
                self.onchain.account.address,
                voter_wallet_address
            );
        } else {
            log::error!(
                "voter is not correctly set, hot: {:?}, expected voter: {:?}",
                self.onchain.account.address,
                voter_wallet_address
            );
        }
        Ok(())
    }

    pub fn generate_random_int_pt(&self) -> u128 {
        let random_u128: u128 = rand::random();
        log::debug!("generated random_int: {}", random_u128);
        return random_u128;
    }
}
