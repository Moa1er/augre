// Main entrypoint.

// Directives.
#![warn(rustdoc::broken_intra_doc_links, rust_2018_idioms, clippy::all)]
#![allow(incomplete_features)]

// Modules.

pub mod base;
pub mod services;

// Imports.

use base::{types::{Void, EnsurableEntity, Mode, RemovableEntity}, config::Config};
use clap::{command, Parser, Subcommand};
use services::{git::Git, gpt::Gpt, cria::Cria};
use termimad::MadSkin;
use yansi::Paint;

use crate::services::{docker::Docker, gpt, model::Model};

// Commands.

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The path to the data directory.
    #[arg(short, long, default_value = ".augre")]
    data_path: String,

    /// The default operation mode.
    #[arg(short, long, default_value = "openai")]
    mode: Mode,

    /// Whether to skip the confirmation prompt.
    #[clap(long = "yes", short = 'y', action)]
    skip_confirm: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Performs a code review from parent_branche(arg1) to child_branch(arg2)
    Review {
        original_branch_name: String,
        working_branch_name: String,
        gpt_model: Option<String>,
    },

    /// Performs description for making PR from parent_branche(arg1) to child_branch(arg2)
    PRDescription{
        original_branch_name: String,
        working_branch_name: String,
        gpt_model: Option<String>,
    },

    ///Performs description for making PR from parent_branche(arg1) to child_branch(arg2) + reviews it
    PRAndReview{
        original_branch_name: String,
        working_branch_name: String,
        gpt_model: Option<String>,
    },

    /// Gives you a comment for the last changes for your future commit
    CommitMessage{
        gpt_model: Option<String>,
    },

    /// Gives a response to the specified prompt.
    Ask {
        /// The prompt to respond to.
        prompt: String,
    },

    /// Stop all of the background services.
    Stop,
}

// Entrypoint.

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(err) = start(args).await {
        eprintln!("{}: {}", Paint::red("ERROR"), err);
        std::process::exit(1);
    }
}

async fn start(args: Args) -> Void {
    let config = base::config::Config::new(&args.data_path, args.mode)?;
    let confirm = !args.skip_confirm;

    match args.command {
        Some(Command::Review{original_branch_name, working_branch_name, gpt_model }) => review(&config, confirm, &original_branch_name, &working_branch_name, gpt_model).await?,
        Some(Command::PRDescription{ original_branch_name, working_branch_name, gpt_model }) => pr_description(&config, confirm, &original_branch_name, &working_branch_name, gpt_model).await?,
        Some(Command::PRAndReview{ original_branch_name, working_branch_name, gpt_model }) => pr_description_and_review(&config, confirm, &original_branch_name, &working_branch_name, gpt_model).await?,
        Some(Command::CommitMessage{ gpt_model }) => commit_msg(&config, confirm, gpt_model).await?,
        Some(Command::Ask { prompt }) => ask(&config, confirm, &prompt).await?,
        Some(Command::Stop) => stop(&config, confirm).await?,
        None => return Err(anyhow::anyhow!("No command specified.")),
    }

    Ok(())
}

async fn pr_description_and_review(config: &Config, confirm: bool, original_branch_name: &str, working_branch_name: &str, gpt_model: Option<String>) -> Void {
    println!();
    println!("Giving the description first: ");
    pr_description(&config, confirm, &original_branch_name, &working_branch_name, gpt_model.clone()).await?;
    
    println!();
    println!("Giving the review: ");
    review(&config, confirm, &original_branch_name, &working_branch_name, gpt_model.clone()).await?;
    Ok(())
}

async fn commit_msg(
    config: &Config, 
    confirm: bool, 
    gpt_model: Option<String>,
)-> Void {
    println!();

    maybe_prepare_local(config, confirm).await?;

    let git = Git::default();
    let gpt = Gpt::new(&config.openai_endpoint, &config.openai_key, config.mode);

    git.ensure(confirm).await?;
    gpt.ensure(confirm).await?;

    println!();

    // print!("Getting diffs between ", original_branch_name, " and ", working_branch_name);
    print!("{}", format!("Getting code diff from last changes..."));
    let diff = Git::diff().await?;
    println!("{}", Paint::green("✔️"));

    print!("Getting commit msg ...");
    let response = gpt.make_req_with_prompt(&diff, gpt_model, "commit_msg").await?.trim().to_string();
    println!("{}", Paint::green("✔️"));

    println!();
    println!("Commit msg is:"); 
    let skin = MadSkin::default();
    skin.print_text(&response);

    Ok(())
}


async fn pr_description(
    config: &Config, 
    confirm: bool, 
    original_branch_name: &str,
    working_branch_name: &str, 
    gpt_model: Option<String>
    ) -> Void {
    println!();

    maybe_prepare_local(config, confirm).await?;

    let git = Git::default();
    let gpt = Gpt::new(&config.openai_endpoint, &config.openai_key, config.mode);

    git.ensure(confirm).await?;
    gpt.ensure(confirm).await?;

    println!();

    // print!("Getting diffs between ", original_branch_name, " and ", working_branch_name);
    print!("{}", format!("Getting diffs between {} and {}", original_branch_name, working_branch_name));
    let diff = Git::diff_custom(original_branch_name, working_branch_name).await?;
    println!("{}", Paint::green("✔️"));

    print!("Getting description ...");
    let response = gpt.make_req_with_prompt(&diff, gpt_model, "description").await?.trim().to_string();
    println!("{}", Paint::green("✔️"));

    println!();
    println!("Description is:"); 

    let skin = MadSkin::default();
    skin.print_text(&response);

    Ok(())
}

async fn review(
    config: &Config, 
    confirm: bool, 
    original_branch_name: &str,
    working_branch_name: &str, 
    gpt_model: Option<String>
    ) -> Void {
    println!();

    maybe_prepare_local(config, confirm).await?;

    let git = Git::default();
    let gpt = Gpt::new(&config.openai_endpoint, &config.openai_key, config.mode);

    git.ensure(confirm).await?;
    gpt.ensure(confirm).await?;

    println!();

    print!("Getting diff ...");
    let diff = Git::diff_custom(original_branch_name, working_branch_name).await?;
    println!("{}", Paint::green("✔️"));

    print!("Getting review ...");
    let response = gpt.make_req_with_prompt(&diff, gpt_model, "review").await?.trim().to_string();
    println!("{}", Paint::green("✔️"));

    println!();
    println!("Review is:"); 
    let skin = MadSkin::default();
    skin.print_text(&response);

    Ok(())
}

async fn ask(config: &Config, confirm: bool, prompt: &str) -> Void {
    println!();

    maybe_prepare_local(config, confirm).await?;

    let gpt = Gpt::new(&config.openai_endpoint, &config.openai_key, config.mode);
    gpt.ensure(confirm).await?;

    println!();

    println!("Getting response ...");
    let response = gpt.ask(prompt).await?.trim().to_string();
    println!("{}", Paint::green("✔️"));

    println!();

    let skin = MadSkin::default();
    skin.print_text(&response);

    Ok(())
}

async fn stop(config: &Config, confirm: bool) -> Void {
    let cria = Cria::new(&config.model_path, &config.data_path, config.mode, config.cria_port);

    cria.remove(confirm).await?;

    Ok(())
}

async fn maybe_prepare_local(config: &Config, confirm: bool) -> Void {
    if config.mode == Mode::LocalCpu || config.mode == Mode::LocalGpu {
        let docker = Docker::default();
        let model = Model::new(&config.model_path, &config.model_url);
        let cria = Cria::new(&config.model_path, &config.data_path, config.mode, config.cria_port);

        docker.ensure(confirm).await?;
        model.ensure(confirm).await?;
        cria.ensure(confirm).await?;
    }

    Ok(())
}