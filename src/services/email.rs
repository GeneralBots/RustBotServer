
use jmap_client::{
    client::Client,
    core::query::Filter,
    email::{self, Property},
    mailbox::{self, Role},
};


pub async fn list_emails(
) -> Result<Vec<email::Email>, jmap_client::Error> {

    // 1. Authenticate with JMAP server
    let client = Client::new()
        .credentials(("test@", ""))
        .connect("https://mail/jmap/")
        .await
        .unwrap();

    let inbox_id = client
        .mailbox_query(
            mailbox::query::Filter::role(Role::Inbox).into(),
            None::<Vec<_>>,
        )
        .await
        .unwrap()
        .take_ids()
        .pop()
        .unwrap();

    let email_id = client
        .email_query(
            Filter::and([
                //            email::query::Filter::subject("test"),
                email::query::Filter::in_mailbox(&inbox_id),
                //          email::query::Filter::has_keyword("$draft"),
            ])
            .into(),
            [email::query::Comparator::from()].into(),
        )
        .await
        .unwrap()
        .take_ids()
        .pop()
        .unwrap();

    // Fetch message
    let email = client
        .email_get(
            &email_id,
            [Property::Subject, Property::Preview, Property::Keywords].into(),
        )
        .await
        .unwrap();

    let mut emails = client
        .email_query(
            Filter::and([email::query::Filter::in_mailbox(inbox_id)])
                .into(),
            [email::query::Comparator::from()].into(),
        )
        .await?;

    let email_ids = emails.take_ids();
    let mut email_list = Vec::new();

    for email_id in email_ids {
        if let Some(email) = client
            .email_get(
                &email_id,
                [Property::Subject, Property::Preview, Property::Keywords].into(),
            )
            .await?
        {
            email_list.push(email);
        }
    }

    Ok(email_list)
    
}