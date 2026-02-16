use crate::client::ApiClient;
use crate::error::Result;
use super::types::{Person, CreateContactRequest, Name, EmailAddress, PhoneNumber, Organization};

pub struct CreateContactParams {
    pub given_name: String,
    pub family_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub org_name: Option<String>,
    pub org_title: Option<String>,
}

pub async fn create_contact(client: &ApiClient, params: CreateContactParams) -> Result<Person> {
    let mut request = CreateContactRequest {
        names: vec![Name {
            given_name: Some(params.given_name),
            family_name: params.family_name,
            display_name: None,
            display_name_last_first: None,
            metadata: None,
        }],
        email_addresses: Vec::new(),
        phone_numbers: Vec::new(),
        organizations: Vec::new(),
    };

    if let Some(email) = params.email {
        request.email_addresses.push(EmailAddress {
            value: Some(email),
            email_type: Some("work".to_string()),
            formatted_type: None,
            metadata: None,
        });
    }

    if let Some(phone) = params.phone {
        request.phone_numbers.push(PhoneNumber {
            value: Some(phone),
            phone_type: Some("work".to_string()),
            formatted_type: None,
            metadata: None,
        });
    }

    if params.org_name.is_some() || params.org_title.is_some() {
        request.organizations.push(Organization {
            name: params.org_name,
            title: params.org_title,
            department: None,
            metadata: None,
        });
    }

    client.post("/people:createContact", &request).await
}

pub async fn delete_contact(client: &ApiClient, resource_name: &str) -> Result<()> {
    let path = format!("/{}:deleteContact", resource_name);
    client.delete(&path).await
}
