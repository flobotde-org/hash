use core::{assert_matches::assert_matches, error::Error};

use hash_graph_authorization::policies::principal::{PrincipalId, team::TeamId};
use hash_graph_postgres_store::permissions::PrincipalError;
use pretty_assertions::assert_eq;
use type_system::web::OwnedById;
use uuid::Uuid;

use crate::DatabaseTestWrapper;

#[tokio::test]
async fn create_web() -> Result<(), Box<dyn Error>> {
    let mut db = DatabaseTestWrapper::new().await;
    let mut client = db.client().await?;

    let web_id = client.create_web(None).await?;
    assert!(client.is_web(web_id).await?);

    let retrieved = client.get_web(web_id).await?.expect("Web should exist");
    assert_eq!(retrieved.id, web_id);

    Ok(())
}

#[tokio::test]
async fn create_web_with_id() -> Result<(), Box<dyn Error>> {
    let mut db = DatabaseTestWrapper::new().await;
    let mut client = db.client().await?;

    let id = Uuid::new_v4();
    let web_id = client.create_web(Some(id)).await?;
    assert_eq!(web_id, OwnedById::new(id));
    assert!(client.is_web(web_id).await?);

    Ok(())
}

#[tokio::test]
async fn delete_web() -> Result<(), Box<dyn Error>> {
    let mut db = DatabaseTestWrapper::new().await;
    let mut client = db.client().await?;

    let web_id = client.create_web(None).await?;
    assert!(client.is_web(web_id).await?);

    client.delete_web(web_id).await?;
    assert!(!client.is_web(web_id).await?);

    Ok(())
}

#[tokio::test]
async fn create_web_with_duplicate_id() -> Result<(), Box<dyn Error>> {
    let mut db = DatabaseTestWrapper::new().await;
    let mut client = db.client().await?;

    let web_id = client.create_web(Some(Uuid::new_v4())).await?;
    let result = client.create_web(Some(*web_id.as_uuid())).await;
    drop(client);

    assert_matches!(
        result.expect_err("Creating a web with duplicate ID should fail").current_context(),
        PrincipalError::PrincipalAlreadyExists { id } if *id == PrincipalId::Team(TeamId::Web(web_id))
    );

    Ok(())
}

#[tokio::test]
async fn get_non_existent_web() -> Result<(), Box<dyn Error>> {
    let mut db = DatabaseTestWrapper::new().await;
    let client = db.client().await?;

    let non_existent_id = OwnedById::new(Uuid::new_v4());
    let result = client.get_web(non_existent_id).await?;

    assert!(
        result.is_none(),
        "Getting a non-existent web should return None"
    );

    Ok(())
}

#[tokio::test]
async fn delete_non_existent_web() -> Result<(), Box<dyn Error>> {
    let mut db = DatabaseTestWrapper::new().await;
    let mut client = db.client().await?;

    let non_existent_id = OwnedById::new(Uuid::new_v4());
    let result = client.delete_web(non_existent_id).await;

    assert_matches!(
        result.expect_err("Deleting a non-existent web should fail").current_context(),
        PrincipalError::PrincipalNotFound { id } if *id == PrincipalId::Team(TeamId::Web(non_existent_id))
    );

    Ok(())
}
