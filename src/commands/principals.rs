use super::membership::{MembershipArgs, handle_membership_command};
use crate::{config::Config, get_data_stores};
use clap::{Parser, Subcommand};
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use password_hash::PasswordHasher;
use password_hash::SaltString;
use rand::rngs::OsRng;
use rustical_store::auth::{AuthenticationProvider, User, user::PrincipalType};

#[derive(Parser, Debug)]
pub struct PrincipalsArgs {
    #[arg(short, long, env, default_value = "/etc/rustical/config.toml")]
    config_file: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
struct CreateArgs {
    id: String,
    #[arg(value_enum, short, long)]
    principal_type: Option<PrincipalType>,
    #[arg(short, long)]
    name: Option<String>,
    #[arg(long, help = "Ask for password input")]
    password: bool,
}

#[derive(Parser, Debug)]
struct RemoveArgs {
    id: String,
}

#[derive(Parser, Debug)]
struct EditArgs {
    id: String,
    #[arg(long, help = "Ask for password input")]
    password: bool,
    #[arg(
        long,
        help = "Remove password (If you only want to use OIDC for example)"
    )]
    remove_password: bool,
    #[arg(short, long, help = "Change principal displayname")]
    name: Option<String>,
    #[arg(value_enum, short, long, help = "Change the principal type")]
    principal_type: Option<PrincipalType>,
}

#[derive(Debug, Subcommand)]
enum Command {
    List,
    Create(CreateArgs),
    Remove(RemoveArgs),
    Edit(EditArgs),
    Membership(MembershipArgs),
}

pub async fn cmd_principals(args: PrincipalsArgs) -> anyhow::Result<()> {
    let config: Config = Figment::new()
        .merge(Toml::file(&args.config_file))
        .merge(Env::prefixed("RUSTICAL_").split("__"))
        .extract()?;

    let (_, _, _, principal_store, _) = get_data_stores(true, &config.data_store).await?;

    match args.command {
        Command::List => {
            for principal in principal_store.get_principals().await? {
                println!(
                    "{} (displayname={}) [{}]",
                    principal.id,
                    principal.displayname.unwrap_or_default(),
                    principal.principal_type
                );
            }
        }
        Command::Create(CreateArgs {
            id,
            principal_type,
            name,
            password,
        }) => {
            let salt = SaltString::generate(OsRng);
            let password = if password {
                println!("Enter your password:");
                let password = rpassword::read_password()?;
                Some(
                    argon2::Argon2::default()
                        .hash_password(password.as_bytes(), &salt)
                        .unwrap()
                        .to_string()
                        .into(),
                )
            } else {
                None
            };
            principal_store
                .insert_principal(
                    User {
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
        Command::Remove(RemoveArgs { id }) => {
            principal_store.remove_principal(&id).await?;
            println!("Principal {id} removed");
        }
        Command::Edit(EditArgs {
            id,
            remove_password,
            password,
            name,
            principal_type,
        }) => {
            let mut principal = principal_store
                .get_principal(&id)
                .await?
                .unwrap_or_else(|| panic!("Principal {id} does not exist"));

            if remove_password {
                principal.password = None;
            }
            if password {
                let salt = SaltString::generate(OsRng);
                println!("Enter your password:");
                let password = rpassword::read_password()?;
                principal.password = Some(
                    argon2::Argon2::default()
                        .hash_password(password.as_bytes(), &salt)
                        .unwrap()
                        .to_string()
                        .into(),
                )
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
        Command::Membership(args) => {
            handle_membership_command(principal_store.as_ref(), args).await?
        }
    }
    Ok(())
}
