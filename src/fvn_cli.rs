#[derive(clap::Subcommand, Clone, Debug)]
pub enum Commands {
    /// check hot wallet address, by default will use ./config/config_fvn.toml
    CheckHotWalletAddress(CheckHotWalletAddress),
    /// check hot wallet gas balance, need gas fee to vote
    CheckGasBalance(CheckGasBalance),
    /// check if hot wallet has registered with a particular voter wallet
    CheckRegistration(CheckRegistration),
    /// register voter address
    Register(Register),
    /// think a number and vote once
    DeepseekFheVote(DeepSeekApiKey),
    /// check voting rewards
    CheckVoteRewards(CheckVoteRewards),
    /// check voting tx history on the explore
    CheckVote(CheckVote),
    /// check voting tx history on the explore
    CheckRound(CheckRound),
}

#[derive(clap::Args, Clone, Debug)]
pub struct CheckGasBalance {
    pub hot_wallet_address: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct CheckRegistration {
    pub hot_wallet_address: Option<String>,
    pub voter_wallet_address: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct Register {
    pub voter_wallet_address: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct DeepSeekApiKey {
    pub deepseek_api_key: String,
    pub fhe_public_key_fp: String
}

#[derive(clap::Args, Clone, Debug)]
pub struct VoteOnceRandom {}

#[derive(clap::Args, Clone, Debug)]
pub struct VoteLoopRandom {}

#[derive(clap::Args, Clone, Debug)]
pub struct CheckVoteRewards {
    pub voter_wallet_address: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct CheckVote {}

#[derive(clap::Args, Clone, Debug)]
pub struct CheckRound {}

#[derive(clap::Args, Clone, Debug)]
pub struct CheckHotWalletAddress {}

#[derive(clap::Parser, Clone, Debug)]
#[command(author = "Mind Network", version = "0.1.0", about = "FHE Randgen Voter Node Cli", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(default_value = "./config/config_randgen.toml")]
    #[arg(long)]
    /// fvn config file, contains all the config to run fvn
    pub node_config_file: String,

    #[arg(long, value_enum, default_value_t = mind_sdk_cli::CliLogLevel::Info)] //, rename_all = "UPPER")]
    /// control level of print, useful for debug, default is info
    pub log_level: mind_sdk_cli::CliLogLevel,

    #[arg(long)]
    /// fvn wallet private key is needed if to load a different wallet from config_fvn.toml to sign the message onchain, by default load from ./config/config_fvn.toml
    pub hot_wallet_private_key: Option<String>,

    //#[arg(long)]
    /// fvn will randomly generate a number and do FHE encryption to vote, you can specific a given number instead of random, useful to integrate with other use case
    //pub vote: Option<u128>,

    #[command(subcommand)]
    pub command: Commands,
}
