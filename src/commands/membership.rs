use clap::{Parser, Subcommand};
use rustical_store::auth::AuthenticationProvider;

#[derive(Debug, Parser)]
pub struct AssignArgs {
    id: String,
    #[arg(long, help = "The principal to assign a membership to (e.g. a group)")]
    to: String,
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
    command: MembershipCommand,
}

pub async fn handle_membership_command(
    user_store: impl AuthenticationProvider,
    MembershipArgs { command }: MembershipArgs,
) -> anyhow::Result<()> {
    let id = match &command {
        MembershipCommand::Assign(AssignArgs { id, .. }) => id,
        MembershipCommand::Remove(RemoveArgs { id, .. }) => id,
        MembershipCommand::List(ListArgs { id }) => id,
    };
    let mut principal = user_store
        .get_principal(id)
        .await?
        .unwrap_or_else(|| panic!("Principal {id} does not exist"));

    match command {
        MembershipCommand::Assign(AssignArgs { to, .. }) => {
            if principal.memberships.contains(&to) {
                println!("Principal is already member of {to}");
                return Ok(());
            }
            principal.memberships.push(to);
            user_store.insert_principal(principal, true).await?;
            println!("Membership assigned");
        }
        MembershipCommand::Remove(RemoveArgs { to, .. }) => {
            principal.memberships.retain(|principal| principal != &to);
            user_store.insert_principal(principal, true).await?;
            println!("Membership removed");
        }
        MembershipCommand::List(ListArgs { .. }) => {
            for membership in principal.memberships {
                println!("{membership}");
            }
        }
    }
    Ok(())
}
