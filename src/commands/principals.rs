use super::membership::MembershipArgs;
use crate::{config::Config, get_data_stores, membership::cmd_membership};
use clap::{Parser, Subcommand};
use password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
use rustical_store::{
    Secret,
    auth::{AuthenticationProvider, Principal, PrincipalType},
};

#[derive(Parser, Debug)]
pub struct PrincipalsArgs {
    #[command(subcommand)]
    pub command: PrincipalsCommand,
}

#[derive(Parser, Debug)]
pub struct CreateArgs {
    pub id: String,
    #[arg(value_enum, short, long)]
    pub principal_type: Option<PrincipalType>,
    #[arg(short, long)]
    pub name: Option<String>,
    // This argument is just there so that we can create a user password in an integration test
    // environment
    #[arg(long, hide = true)]
    pub for_testing_password_from_arg: Option<String>,
    #[arg(long, help = "Ask for password input")]
    pub password: bool,
}

#[derive(Parser, Debug)]
pub struct RemoveArgs {
    pub id: String,
}

#[derive(Parser, Debug)]
pub struct EditArgs {
    pub id: String,
    // This argument is just there so that we can create a user password in an integration test
    // environment
    #[arg(long, hide = true)]
    pub for_testing_password_from_arg: Option<String>,
    #[arg(long, help = "Ask for password input")]
    pub password: bool,
    #[arg(
        long,
        help = "Remove password (If you only want to use OIDC for example)"
    )]
    pub remove_password: bool,
    #[arg(short, long, help = "Change principal displayname")]
    pub name: Option<String>,
    #[arg(value_enum, short, long, help = "Change the principal type")]
    pub principal_type: Option<PrincipalType>,
}

#[derive(Debug, Subcommand)]
pub enum PrincipalsCommand {
    List,
    Create(CreateArgs),
    Remove(RemoveArgs),
    Edit(EditArgs),
    Membership(MembershipArgs),
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn cmd_principals(args: PrincipalsArgs, config: Config) -> anyhow::Result<()> {
    let (_, _, _, principal_store, _) = get_data_stores(true, &config.data_store).await?;

    match args.command {
        PrincipalsCommand::List => {
            for principal in principal_store.get_principals().await? {
                println!(
                    "{} (displayname={}) [{}]",
                    principal.id,
                    principal.displayname.unwrap_or_default(),
                    principal.principal_type
                );
            }
        }
        PrincipalsCommand::Create(CreateArgs {
            id,
            principal_type,
            name,
            password,
            for_testing_password_from_arg,
        }) => {
            let password = if let Some(pass) = for_testing_password_from_arg {
                Some(pass)
            } else if password {
                Some(rpassword::read_password()?)
            } else {
                None
            };
            let password = password.map(|password| {
                let salt = SaltString::generate(OsRng);
                Secret::from(
                    argon2::Argon2::default()
                        .hash_password(password.as_bytes(), &salt)
                        .unwrap()
                        .to_string(),
                )
            });

            principal_store
                .insert_principal(
                    Principal {
                        id,
                        displayname: name,
                        principal_type: principal_type.unwrap_or_default(),
                        password,
                        memberships: vec![],
                    },
                    false,
                )
                .await?;
            println!("Principal created");
        }
        PrincipalsCommand::Remove(RemoveArgs { id }) => {
            principal_store.remove_principal(&id).await?;
            println!("Principal {id} removed");
        }
        PrincipalsCommand::Edit(EditArgs {
            id,
            remove_password,
            password,
            name,
            principal_type,
            for_testing_password_from_arg,
        }) => {
            let mut principal = principal_store
                .get_principal(&id)
                .await?
                .unwrap_or_else(|| panic!("Principal {id} does not exist"));

            if remove_password {
                principal.password = None;
            }

            let password = if let Some(pass) = for_testing_password_from_arg {
                Some(pass)
            } else if password {
                Some(rpassword::read_password()?)
            } else {
                None
            };
            let password = password.map(|password| {
                let salt = SaltString::generate(OsRng);
                Secret::from(
                    argon2::Argon2::default()
                        .hash_password(password.as_bytes(), &salt)
                        .unwrap()
                        .to_string(),
                )
            });
            if password.is_some() {
                principal.password = password;
            }

            if name.is_some() {
                principal.displayname = name;
            }
            if let Some(principal_type) = principal_type {
                principal.principal_type = principal_type;
            }
            principal_store.insert_principal(principal, true).await?;
            println!("Principal {id} updated");
        }
        PrincipalsCommand::Membership(args) => {
            cmd_membership(principal_store.as_ref(), args).await?;
        }
    }
    Ok(())
}
