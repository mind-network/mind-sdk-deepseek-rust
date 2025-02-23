impl crate::Fvn {
    pub async fn new(node_config_file: String) -> Result<Self, mind_sdk_util::MindError> {
        let config: crate::fvn_config::FvnConfig =
            crate::fvn_config::FvnConfig::new_from_file(node_config_file.clone())?;
        let basic_config: mind_sdk_config::BasicConfig =
            mind_sdk_config::BasicConfig::new_from_file(config.basic_config_file.clone())?;
        let rewards_contract_address = basic_config.rewards_contract_address;
        let fhekeyregistry_contract_address = basic_config.fhekeyregistry_contract_address;
        let subnet_controller_contract_address = basic_config.subnet_controller_contract_address;
        let onchain = mind_sdk_chain::function::OnChainFunction::new(
            config.fvn_wallet_private_key.clone(),
            basic_config.rpc_url_read.clone(),
            rewards_contract_address.clone(),
            fhekeyregistry_contract_address.clone(),
            subnet_controller_contract_address.clone(),
        )
        .await?;
        let given_random_int_pt = None;
        let fhe_public_key_local_fp = "".to_string();
        let fhe: Option<mind_sdk_fhe::FheInt> = None;
        let fvn_contract = crate::fvn_contract::get_fvn_contract_instance(
            &basic_config.randgen_contract_address,
            &onchain.rpc.provider,
        )?;
        let fvn_round_contract = crate::fvn_round_contract::get_fvn_round_contract_instance(
            &basic_config.randgen_round_contract_address,
            &onchain.rpc.provider,
        )?;

        Ok(Self {
            node_config_file: node_config_file.clone(),
            config: config.clone(),
            basic_config: basic_config.clone(),
            onchain: onchain.clone(),
            fhe_public_key_local_fp: fhe_public_key_local_fp,
            given_random_int_pt: given_random_int_pt,
            fhe: fhe,
            fvn_contract: fvn_contract,
            fvn_round_contract: fvn_round_contract,
        })
    }

    pub async fn update_config(&mut self) -> Result<(), mind_sdk_util::MindError> {
        self.onchain.wallet_private_key = self.config.fvn_wallet_private_key.clone();
        self.onchain.rpc_url = self.basic_config.rpc_url_read.clone();
        self.onchain.update_config().await?;

        // must reset here to avoid "Missing signing credential for" error
        self.fvn_contract = crate::fvn_contract::get_fvn_contract_instance(
            &self.basic_config.randgen_contract_address,
            &self.onchain.rpc.provider,
        )?;
        self.fvn_round_contract = crate::fvn_round_contract::get_fvn_round_contract_instance(
            &self.basic_config.randgen_round_contract_address,
            &self.onchain.rpc.provider,
        )?;
        Ok(())
    }

    pub fn set_hot_wallet_private_key(&mut self, hot_wallet_private_key: String) {
        self.config.fvn_wallet_private_key = hot_wallet_private_key;
    }
}

///////////////////////////////////////////////////////////////////////////
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FvnConfig {
    pub basic_config_file: String,
    pub fvn_wallet_private_key: String, // hot_wallet_key: String,
    pub fhe_key_type: String,
    pub dir_keys: String,
    pub dir_data_fvn: String,
    pub poll_interval: u64,
}

impl FvnConfig {
    pub fn new_from_file(fp_config: String) -> Result<Self, mind_sdk_util::MindError> {
        log::debug!("read node config file: {:?}", &fp_config);
        let content = std::fs::read_to_string(fp_config.clone())?;
        return FvnConfig::new_from_string(content);
    }

    pub fn new_from_string(content: String) -> Result<Self, mind_sdk_util::MindError> {
        let config = toml::from_str(&content).map_err(|e| {
            log::error!("TOML parse error: {:#?}", e);
            mind_sdk_util::MindError::TomlParseError(e.to_string())
        })?;
        Ok(config)
    }
}
