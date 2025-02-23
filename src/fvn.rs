use alloy::sol_types::{SolValue};
use mind_sdk_chain::fhe::FvnNodeInterface;

////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct Fvn {
    pub config: crate::fvn_config::FvnConfig,
    pub basic_config: mind_sdk_config::BasicConfig,
    pub node_config_file: String,
    pub onchain: mind_sdk_chain::function::OnChainFunction,
    pub given_random_int_pt: Option<u128>,
    pub fhe_public_key_local_fp: String,
    pub fhe: Option<mind_sdk_fhe::FheInt>,
    pub fvn_contract: crate::fvn_contract::FvnContractInstance,
    pub fvn_round_contract: crate::fvn_round_contract::FvnRoundContractInstance,
}

impl mind_sdk_chain::fhe::FvnNodeInterface for Fvn {
    async fn download_fhekey_if_not_exist(&mut self) -> Result<(), mind_sdk_util::MindError> {
        let fhe_public_key_data = self.fvn_contract.fheKeySetID().call().await?;
        let fhekeyset_id = fhe_public_key_data._0;
        let fp_dir = self.config.dir_data_fvn.clone();
        let fhe_key_type = self.config.fhe_key_type.clone();
        let _pk_fp = self
            .onchain
            .get_fhe_publickey(
                fhekeyset_id,
                fp_dir,
                self.basic_config.clone(),
                fhe_key_type,
            )
            .await?;
        self.fhe_public_key_local_fp = _pk_fp;
        Ok(())
    }

    async fn set_fhe(&mut self) -> Result<(), mind_sdk_util::MindError> {
        self.download_fhekey_if_not_exist().await?;
        let fhe: mind_sdk_fhe::FheInt =
            mind_sdk_fhe::FheInt::new_from_public_key_local(&self.fhe_public_key_local_fp);
        self.fhe = Some(fhe);
        log::debug!("set_fhe: {:?} {:?}", self.fhe.is_some(), self.fhe);
        Ok(())
    }

    fn fhe_encrypt(&self, random_u128: u128) -> Result<String, mind_sdk_util::MindError> {
        match &self.fhe {
            Some(fhe) => {
                let random_ct: tfhe::integer::RadixCiphertext =
                    mind_sdk_fhe::fhe_client::encrypt(fhe, "u8", random_u128);
                let fhe_content: String = mind_sdk_fhe::io::serialize_base64(random_ct)?;
                log::debug!(
                    "encrypt ({:?}) and get ciphertext with size: {}",
                    random_u128,
                    fhe_content.len()
                );
                return Ok(fhe_content);
            }
            None => {
                let error = mind_sdk_util::MindError::FheNotInit("".to_string());
                return Err(error);
            }
        }
    }

    async fn submit_fhe_encrypted(
        &self,
        fhe_content: String,
    ) -> Result<alloy::rpc::types::TransactionReceipt, mind_sdk_util::MindError> {
        log::debug!("submit ciphertext: {}", &fhe_content.len());
        let response = mind_sdk_io::upload_ciphertext(
            &fhe_content,
            &self.config.fvn_wallet_private_key,
            self.basic_config.subnet_id as usize,
            &self.basic_config.upload_url,
        )
        .await?;
        log::debug!("{:#?}", response);
        if response.status().is_success() {
            /* let digest = sha3::Keccak256::new_with_prefix(fhe_content.clone());
                       let file_digest = digest.clone().finalize();
                       let filename = hex::encode(file_digest);
                       log::debug!("filename: {:#?}, len: {}", filename, filename.len());
                       log::debug!("upload_url: {:#?}", &self.basic_config.upload_url);
            */
            let url_response: mind_sdk_io::Reply = response.json().await?;
            log::debug!("url: {:#?}", url_response);
            let url = url_response.url;
            log::debug!("url: {:#?}", url);

            let gas_amount = self.basic_config.gas_amount;
            let tx_receipt = self.submit_random_ct(url, gas_amount).await?;
            return Ok(tx_receipt);
        } else {
            log::error!("{:#?}", &response);
            let es = format!("cipher upload error: {:#?}", &response);
            let error = mind_sdk_util::MindError::CloudStroageCipherUploadError(es.clone());
            return Err(error);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////
impl crate::Fvn {
    pub async fn submit_random_ct(
        &self,
        ct_url: String,
        new_gas_amount: u64,
    ) -> Result<alloy::rpc::types::TransactionReceipt, mind_sdk_util::MindError> {
        // Step 0: Set up call builder
        let contract_call_builder = self.fvn_contract.submitRandomCt(ct_url.abi_encode().into());

        // Step 1: Set up the transaction with the gas amount and wallet address
        let tx = contract_call_builder
            .clone()
            .gas(new_gas_amount)
            .from(self.onchain.account.address);

        // Step 2: Run the simulation
        match tx.call().await {
            Ok(simulation_result) => {
                log::debug!("Simulation successful: {:?}", simulation_result);

                // Only proceed with sending the transaction if the simulation is successful
                let send_result = tx.send().await?;

                // Step 3: Wait for the transaction receipt
                let tx_receipt = send_result.get_receipt().await?;
                //log::info!("Confirmed TX: {:?}", tx_receipt);
                log::info!("Confirmed TX: status: {:?}, hash: {:?}, gas: {:?}, block: {:?}", &tx_receipt.status(), &tx_receipt.transaction_hash, &tx_receipt.gas_used, &tx_receipt.block_number.unwrap_or(0));
                Ok(tx_receipt)
            }
            Err(e) => {
                return Err(self.process_contract_call_error(e).await);
            },
        }
    }
}



///////////////////////////////////////////////////////////////////////////
impl crate::Fvn {
    // for randgen
    pub async fn set(
        node_wallet_private_key: String,
        fp_config_template: String,
        int: u128,
    ) -> Result<Self, mind_sdk_util::MindError> {
        let mut fvn = crate::Fvn::new(fp_config_template).await?;
        fvn.config.fvn_wallet_private_key = node_wallet_private_key.clone();
        fvn.update_config().await?;
        fvn.given_random_int_pt = Some(int);
        return Ok(fvn);
    }

    pub async fn run_once(
        &mut self, prediction: u128
    ) -> Result<alloy::rpc::types::TransactionReceipt, mind_sdk_util::MindError> {
        let isvoterready = self.check_if_voter_ready().await?;
        match isvoterready {
            true => {
                let pt = prediction;

                if self.fhe.is_none() {
                    let _ = self.set_fhe().await;
                }

                log::debug!(
                    "hot: {:?}, to_encrypt: {:?}",
                    self.onchain.account.address,
                    pt
                );
                let ct = self.fhe_encrypt(pt)?;
                let result: alloy::rpc::types::TransactionReceipt =
                    self.submit_fhe_encrypted(ct).await?;
                return Ok(result);
            }
            false => {
                let es = format!("voter is not ready: {:#?}", &self.onchain.account.address);
                log::error!("{}", &es);
                let error = mind_sdk_util::MindError::VoterNotReady(es);
                return Err(error);
            }
        }
    }


    pub async fn run_once_with_deepseek_prediction(&mut self, fhe_public_key_fp: String) -> Result<alloy::rpc::types::TransactionReceipt, mind_sdk_util::MindError> {
        let client = deepseek_rs::DeepSeekClient::default().unwrap();
        let request = deepseek_rs::client::chat_completions::request::RequestBody::new_messages(vec![
            deepseek_rs::client::chat_completions::request::Message::new_user_message("Please predict BTC price in next 7 days, return must be a positive integer".to_string())
        ]).with_model(deepseek_rs::client::chat_completions::request::Model::DeepSeekReasoner);
        let response = client.chat_completions(request).await.unwrap();
        //println!("Reasoning: {}", response.choices[0].message.reasoning_content.unwrap());
        //println!("Answer: {}", response.choices[0].message.content.unwrap());
        let deepseek_prediction = match response.choices[0].clone().message.content.unwrap().parse::<u128>() {
            Ok(prediction) => prediction,
            Err(_) => 0,
        };
        let fhe: mind_sdk_fhe::FheInt = mind_sdk_fhe::FheInt::new_from_public_key_local(&fhe_public_key_fp);
        let ciphertext = mind_sdk_fhe::fhe_client::encrypt(&fhe, "u8", deepseek_prediction.clone());
        let ciphertext_str: String = mind_sdk_fhe::io::serialize_base64(ciphertext)?;
        let result: alloy::rpc::types::TransactionReceipt = self.submit_fhe_encrypted(ciphertext_str).await?;
        return Ok(result);
    }
}
