use clap::{Parser, Subcommand};
use rustical_store::auth::AuthenticationProvider;

#[derive(Debug, Parser)]
pub struct AssignArgs {
    pub id: String,
    #[arg(long, help = "The principal to assign a membership to (e.g. a group)")]
    pub to: String,
}

#[derive(Debug, Parser)]
pub struct RemoveArgs {
    id: String,
    #[arg(long, help = "The membership to remove")]
    to: String,
}

#[derive(Debug, Parser)]
pub struct ListArgs {
    id: String,
}

#[derive(Debug, Subcommand)]
pub enum MembershipCommand {
    Assign(AssignArgs),
    Remove(RemoveArgs),
    List(ListArgs),
}

#[derive(Parser, Debug)]
pub struct MembershipArgs {
    #[command(subcommand)]
    pub command: MembershipCommand,
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn cmd_membership(
    user_store: &impl AuthenticationProvider,
    MembershipArgs { command }: MembershipArgs,
) -> anyhow::Result<()> {
    let id = match &command {
        MembershipCommand::Assign(AssignArgs { id, .. })
        | MembershipCommand::Remove(RemoveArgs { id, .. })
        | MembershipCommand::List(ListArgs { id }) => id,
    };

    match &command {
        MembershipCommand::Assign(AssignArgs { to, .. }) => {
            user_store.add_membership(id, to).await?;
            println!("Membership assigned");
        }
        MembershipCommand::Remove(RemoveArgs { to, .. }) => {
            user_store.remove_membership(id, to).await?;
            println!("Membership removed");
        }
        MembershipCommand::List(ListArgs { .. }) => {
            let principal = user_store
                .get_principal(id)
                .await?
                .unwrap_or_else(|| panic!("Principal {id} does not exist"));
            for membership in principal.memberships() {
                println!("{membership}");
            }
        }
    }
    Ok(())
}
