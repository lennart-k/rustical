use crate::{config::Config, get_data_stores};
use clap::{Parser, Subcommand, ValueEnum};
use rustical_store::{Access, ShareStore};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AccessArg {
    Read,
    Write,
}

impl From<AccessArg> for Access {
    fn from(value: AccessArg) -> Self {
        match value {
            AccessArg::Read => Self::Read,
            AccessArg::Write => Self::Write,
        }
    }
}

#[derive(Debug, Parser)]
pub struct AddArgs {
    #[arg(help = "The owner of the collection")]
    pub owner: String,
    #[arg(help = "The id of the collection")]
    pub id: String,
    #[arg(help = "The principal to share with: its id or its e-mail address")]
    pub sharee: String,
    #[arg(value_enum, long, default_value = "read")]
    pub access: AccessArg,
}

#[derive(Debug, Parser)]
pub struct ListArgs {
    pub owner: String,
    pub id: String,
}

#[derive(Debug, Parser)]
pub struct RemoveArgs {
    pub owner: String,
    pub id: String,
    #[arg(help = "The principal id of the sharee")]
    pub sharee: String,
}

#[derive(Debug, Subcommand)]
pub enum ShareCommand {
    #[command(about = "Share, or change the access of an existing share")]
    Add(AddArgs),
    List(ListArgs),
    Remove(RemoveArgs),
}

#[derive(Debug, Subcommand)]
pub enum CollectionKind {
    Calendar {
        #[command(subcommand)]
        command: ShareCommand,
    },
    Addressbook {
        #[command(subcommand)]
        command: ShareCommand,
    },
}

#[derive(Debug, Parser)]
pub struct ShareArgs {
    #[command(subcommand)]
    pub kind: CollectionKind,
}

#[allow(clippy::missing_errors_doc)]
pub async fn cmd_share(args: ShareArgs, config: Config) -> anyhow::Result<()> {
    let (_, _, _, _, share_store, _) = get_data_stores(true, &config.data_store).await?;
    match args.kind {
        CollectionKind::Calendar { command } => match command {
            ShareCommand::Add(AddArgs {
                owner,
                id,
                sharee,
                access,
            }) => {
                let share = share_store
                    .share_calendar(&owner, &id, &sharee, access.into(), None)
                    .await?;
                println!(
                    "Calendar {owner}/{id} shared with {} as {} ({})",
                    share.sharee,
                    share.name,
                    share.access.as_str()
                );
            }
            ShareCommand::List(ListArgs { owner, id }) => {
                for share in share_store.list_calendar_shares(&owner, &id).await? {
                    println!(
                        "{} {} ({})",
                        share.sharee,
                        share.name,
                        share.access.as_str()
                    );
                }
            }
            ShareCommand::Remove(RemoveArgs { owner, id, sharee }) => {
                share_store.unshare_calendar(&owner, &id, &sharee).await?;
                println!("Share removed");
            }
        },
        CollectionKind::Addressbook { command } => match command {
            ShareCommand::Add(AddArgs {
                owner,
                id,
                sharee,
                access,
            }) => {
                let share = share_store
                    .share_addressbook(&owner, &id, &sharee, access.into(), None)
                    .await?;
                println!(
                    "Addressbook {owner}/{id} shared with {} as {} ({})",
                    share.sharee,
                    share.name,
                    share.access.as_str()
                );
            }
            ShareCommand::List(ListArgs { owner, id }) => {
                for share in share_store.list_addressbook_shares(&owner, &id).await? {
                    println!(
                        "{} {} ({})",
                        share.sharee,
                        share.name,
                        share.access.as_str()
                    );
                }
            }
            ShareCommand::Remove(RemoveArgs { owner, id, sharee }) => {
                share_store
                    .unshare_addressbook(&owner, &id, &sharee)
                    .await?;
                println!("Share removed");
            }
        },
    }
    Ok(())
}
