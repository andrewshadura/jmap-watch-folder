mod settings;

use anyhow::{bail, ensure, Context, Result};

use secret_service::EncryptionType;
use secret_service::SecretService;
use std::{collections::HashMap, error::Error, str};

use crate::settings::Settings;
use jmap_client::email::Property;
use jmap_client::{
    client::Client,
    core::query,
    email,
    mailbox::{query::Filter, Role},
};
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    println!(
        "Config path: {}",
        confy::get_configuration_file_path("jmap-watch-folder", None)?.display()
    );
    let cfg: Settings = confy::load("jmap-watch-folder", None)?;
    println!("Config: {:?}", cfg);

    let username = cfg.username.unwrap_or("".into());
    ensure!(!username.is_empty(), "No username specified");
    let mailbox_name = cfg.mailbox.unwrap_or("".into());
    ensure!(!mailbox_name.is_empty(), "No mailbox specified");

    let ss = SecretService::connect(EncryptionType::Dh).await?;

    let collection = ss.get_default_collection().await?;

    let search_items = ss
        .search_items(HashMap::from([
            ("username", username.as_str()),
            ("protocol", "jmap"),
        ]))
        .await?;

    // retrieve one item, first by checking the unlocked items
    let item = match search_items.unlocked.first() {
        Some(item) => item,
        None => {
            // if there aren't any, check the locked items and unlock the first one
            let locked_item = search_items.locked.first().with_context(|| {
                format!("Missing secret for protocol jmap username {}", username)
            })?;
            locked_item.unlock().await?;
            locked_item
        }
    };

    // retrieve secret from item
    let secret = item.get_secret().await?;
    let attributes = item.get_attributes().await?;
    let server = Url::parse(&match cfg.server {
        Some(s) => s,
        None => attributes
            .get("server")
            .context("Secret doesn't specify server name and there is none in config either")
            .map(|s| {
                if !s.starts_with("https://") && !s.starts_with("http://") {
                    format!("https://{}", s)
                } else {
                    s.into()
                }
            })?,
    })
    .expect("Invalid server name");

    let host = server.host_str().expect("Missing host name?!");

    println!("Retrieved secret: {:?}", str::from_utf8(&secret)?);
    println!("Server name: {:?}", server);

    let token = str::from_utf8(&secret)?;

    let client = Client::new()
        .credentials(token)
        .follow_redirects([host])
        .connect(server.as_str())
        .await?;

    let mailbox_path = mailbox_name.split("/").collect::<Vec<_>>();

    let mut folder_id = None::<String>;

    for folder in mailbox_path {
        let folder = client
            .mailbox_query(
                query::Filter::and([
                    Filter::name(folder),
                    Filter::parent_id(folder_id.as_deref()),
                ])
                .into(),
                None::<Vec<_>>,
            )
            .await?
            .take_ids()
            .pop();
        if folder.is_some() {
            folder_id = folder;
        }
    }
    ensure!(folder_id.is_some(), "Can't find folder {}", mailbox_name);

    println!(
        "Mailbox: {:?}",
        client
            .mailbox_get(folder_id.as_deref().unwrap(), None::<Vec<_>>)
            .await?
    );

    let filter = &cfg.watch[0];

    let cond = [
        filter.as_filters(),
        [email::query::Filter::in_mailbox(
            folder_id.as_deref().unwrap(),
        )]
            .into(),
    ].concat();
    eprintln!("cond = {:?}", cond);

    let email_ids = client
        .email_query(
            query::Filter::and(
                cond,
            )
            .into(),
            None::<Vec<_>>,
        )
        .await?
        .take_ids();

    for email_id in email_ids {
        let email = client
            .email_get(
                &email_id,
                [
                    Property::Subject,
                    Property::Preview,
                    Property::Keywords,
                    Property::BlobId,
                    Property::HasAttachment,
                    Property::From,
                    Property::To,
                    Property::Cc,
                    Property::Bcc,
                    Property::ReceivedAt,
                    Property::Attachments,
                ]
                .into(),
            )
            .await?
            .unwrap();
        println!("Email: {:?}", email);
        let body_id = email.blob_id().unwrap();
        //println!("Raw: {}", str::from_utf8(client.download(body_id).await?.as_slice())?);
        let mut keywords = email.keywords();
        //keywords.retain(|&x| x != "$jmap-watch-folder-seen");
        //keywords.retain(|&x| x != "hasattachment");
        //client.email_set_keywords(&email_id, keywords).await?;
        let attachments = email.attachments().unwrap_or(&[]).iter().len();
        eprintln!("attachments: {:?}", attachments);
        if attachments > 0 {
            //client.email_set_keyword(&email_id, "$hasattachment", true).await?;
            keywords.push("$hasattachment");
            client.email_set_keywords(&email_id, keywords).await?;
        }
    }

    /*

    let mailboxes = client
        .mailbox_query(None::<Filter>, None::<Vec<_>>)
        .await
        .unwrap()
        .take_ids();

    for id in mailboxes {
        println!("Mailbox: {:?}", client.mailbox_get(&id, None::<Vec<_>>).await.unwrap());
    }*/

    Ok(())
}
