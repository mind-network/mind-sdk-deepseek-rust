impl crate::Fvn {
    // for randgen
    pub async fn check_rewards(&self) -> Result<Option<(alloy::primitives::Address, u128)>, mind_sdk_util::MindError> {
        let registered_voter_wallet: Option<alloy::primitives::Address> = self.get_voter_wallet().await?;
        match registered_voter_wallet {
            Some(registered_voter_wallet) => {
                let rewards = self
                    .onchain
                    .check_mind_rewards(self.basic_config.subnet_id as i64, registered_voter_wallet)
                    .await?;
                return Ok(Some((registered_voter_wallet, rewards)));
            }
            None => {
                log::error!("hot wallet is not registered and has no rewards");
                return Ok(None);
            }
        }
    }
}
