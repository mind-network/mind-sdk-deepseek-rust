use clap::Parser;

#[tokio::main]
pub async fn main() -> Result<(), mind_sdk_util::MindError> {
    match main_fn().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn main_fn() -> Result<(), mind_sdk_util::MindError> {
    mind_sdk_util::init_log();
    let cli = mind_sdk_deepseek::fvn_cli::Cli::parse();
    println!("cli: {:#?}", cli);
    let log_level = mind_sdk_cli::get_cli_log_level(cli.log_level);
    log::set_max_level(log_level);

    if !std::path::Path::new(&cli.node_config_file.clone()).exists() {
        let isexist = std::path::Path::new(&cli.node_config_file).exists();
        if isexist == false {
            mind_sdk_util::list_files_in_directory_if_file_not_exist(&cli.node_config_file);
        }
        log::error!("your config file is not exist, created or specific with --node-config-file OTHER_CONFIG_FILE_PATH!")
    }

    let app = "randgen".to_string();
    let mut fvn = mind_sdk_deepseek::Fvn::new(cli.node_config_file.clone()).await?;

    match cli.hot_wallet_private_key {
        Some(hot_wallet_private_key) => {
            fvn.config.fvn_wallet_private_key = hot_wallet_private_key.clone();
            fvn.update_config().await?;
        }
        None => {}
    };

    match &cli.command {
        mind_sdk_deepseek::fvn_cli::Commands::CheckHotWalletAddress(_arg) => {
            let mut r = mind_sdk_cli::cli_default_json_result(app);
            r.command = "check-hot-wallet-address".to_string();
            r.result = fvn.onchain.account.address.to_string();
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
        mind_sdk_deepseek::fvn_cli::Commands::CheckGasBalance(arg) => {
            let mut r = mind_sdk_cli::cli_default_json_result(app);
            r.command = "check-gas-balance".to_string();
            match arg.hot_wallet_address.clone() {
                Some(hot_wallet_address) => {
                    r.arg = format!("hot_wallet: {}", hot_wallet_address);
                    r.result = fvn
                        .onchain
                        .get_eth_balance_by_wallet(&hot_wallet_address)
                        .await?
                        .to_string();
                }
                None => {
                    r.arg = format!("hot_wallet: {}", fvn.onchain.account.address.to_string());
                    r.result = fvn.onchain.get_eth_balance().await?.to_string();
                }
            };
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
        mind_sdk_deepseek::fvn_cli::Commands::CheckRegistration(arg) => {
            let mut r = mind_sdk_cli::cli_default_json_result(app);
            r.command = "check-registration".to_string();
            let hot_wallet_address = match arg.hot_wallet_address.clone() {
                Some(hot_wallet_address_str) => {
                    mind_sdk_chain::account::get_address_by_str(&hot_wallet_address_str)?
                }
                None => fvn.onchain.account.address,
            };
            r.arg = hot_wallet_address.to_string();
            let mut voter_wallet_address = alloy::primitives::Address::ZERO;
            match arg.voter_wallet_address.clone() {
                Some(voter_wallet_address_str) => {
                    voter_wallet_address =
                        mind_sdk_chain::account::get_address_by_str(&voter_wallet_address_str)?;
                }
                None => {}
            };
            if voter_wallet_address == alloy::primitives::Address::ZERO {
                let a = fvn.get_voter_wallet().await?;
                match a {
                    Some(a) => {
                        r.status = true;
                        r.result = a.to_string();
                        r.note = format!(
                            "is_registered: true, hot_wallet: {}, voter_wallet: {}",
                            fvn.onchain.account.address,
                            a.to_string()
                        );
                    }
                    None => {
                        r.status = false;
                        r.note = format!("hot wallet is not registered with any voter wallet");
                    }
                }
            } else {
                let registration_status = fvn
                    .check_hot_voter_registration(&hot_wallet_address, &voter_wallet_address)
                    .await?;
                r.status = registration_status;
                r.note = format!(
                    "is_registered: {}, hot_wallet: {}, voter_wallet: {}",
                    registration_status, hot_wallet_address, voter_wallet_address
                );
            }
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
        mind_sdk_deepseek::fvn_cli::Commands::Register(arg) => {
            let mut r = mind_sdk_cli::cli_default_json_result(app);
            r.command = "register".to_string();
            match arg.voter_wallet_address.clone() {
                Some(voter_wallet_address_str) => {
                    let voter_wallet_address =
                        mind_sdk_chain::account::get_address_by_str(&voter_wallet_address_str)?;
                    let _result = fvn.register_voter_wallet(voter_wallet_address).await?;
                    r.arg = format!(
                        "hot_wallet: {}, voter_wallet: {}",
                        fvn.onchain.account.address.to_string(),
                        voter_wallet_address_str
                    );
                    r.status = true;
                    r.result = "registration successful !".to_string();
                    r.note = format!(
                        "is_registered: true, hot_wallet: {}, voter_wallet: {}",
                        fvn.onchain.account.address, voter_wallet_address_str
                    );
                }
                None => {
                    r.arg = format!(
                        "hot_wallet: {}, voter_wallet: none",
                        fvn.onchain.account.address.to_string()
                    );
                    r.status = false;
                    r.result = "voter_wallet is undefined".to_string();
                    r.note = format!("voter_wallet must provide for registration");
                }
            };
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
        mind_sdk_deepseek::fvn_cli::Commands::DeepseekFheVote(arg) => {
            let tx_receipt = fvn.run_once_with_deepseek_prediction(arg.fhe_public_key_fp.clone()).await?;
            let _gas_used = tx_receipt.gas_used;
            let _status = tx_receipt.status();
            let _block_number = tx_receipt.block_number;
            let _tx_hash: String = tx_receipt.transaction_hash.to_string();

            let mut r = mind_sdk_cli::cli_default_json_result(app);
            r.command = "deepseek-fhe-vote".to_string();
            r.arg = format!(
                "number: {}, hot_wallet: {}",
                arg.deepseek_api_key,
                fvn.onchain.account.address.to_string()
            );
            r.status = _status;
            r.result = _tx_hash.to_string();
            r.note = format!(
                "deepseek predicted btc price:, gas_sued: {}, block_number: {:?}, tx_hash: {}",
                _gas_used, _block_number, _tx_hash
            );
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
        mind_sdk_deepseek::fvn_cli::Commands::CheckVoteRewards(_arg) => {
            let rewards = fvn.check_rewards().await?;
            let mut r = mind_sdk_cli::cli_default_json_result(app);
            r.command = "check-vote-rewards".to_string();
            r.arg = format!("hot_wallet: {}", fvn.onchain.account.address.to_string());
            match rewards {
                Some((voter_wallet_address, rewards)) => {
                    r.status = true;
                    r.result = rewards.to_string();
                    r.note = format!(
                        "hot_wallet: {}, voter_wallet: {}, vote_rewards: {}",
                        fvn.onchain.account.address, voter_wallet_address, rewards
                    );
                    println!("{}", serde_json::to_string_pretty(&r).unwrap());
                    Ok(())
                }
                None => {
                    r.status = false;
                    r.result = "".to_string();
                    r.note = format!("hot_wallet is not registered yet",);
                    println!("{}", serde_json::to_string_pretty(&r).unwrap());
                    Ok(())
                }
            }
        }
        mind_sdk_deepseek::fvn_cli::Commands::CheckVote(_arg) => {
            let mut r = mind_sdk_cli::cli_default_json_result(app);
            r.command = "check-vote".to_string();
            r.arg = fvn.onchain.account.address.to_string();
            r.status = true;
            let hot_wallet_address_str = fvn.onchain.account.address.to_string();
            r.result = format!("check on the explore: testnet: https://explorer-testnet.mindnetwork.xyz/address/{}, mainnet: https://explorer.mindnetwork.xyz/address/{}", &hot_wallet_address_str, &hot_wallet_address_str);
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
        mind_sdk_deepseek::fvn_cli::Commands::CheckRound(_arg) => {
            let mut r = mind_sdk_cli::cli_default_json_result(app);
            r.command = "check-round".to_string();
            r.arg = fvn.onchain.account.address.to_string();
            r.status = true;
            let _hot_wallet_address_str = fvn.check_round().await;
            r.result = format!("see stdout");
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            Ok(())
        }
    }
}
