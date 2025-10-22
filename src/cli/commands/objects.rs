use anyhow::{Context, Result};
use anytype_rs::api::{AnytypeClient, CreateObjectRequest, UpdateObjectRequest};
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Debug, Args)]
pub struct ObjectsArgs {
    #[command(subcommand)]
    pub command: ObjectsCommand,
}

#[derive(Debug, Subcommand)]
pub enum ObjectsCommand {
    /// List objects in a space
    List {
        /// Space ID
        space_id: String,
        /// Limit the number of results
        #[arg(short, long)]
        limit: Option<u32>,
    },
    /// Get details of a specific object
    Get {
        /// Space ID where the object exists
        space_id: String,
        /// Object ID to retrieve
        object_id: String,
    },
    /// Create a new object in a space
    Create {
        /// Space ID where the object will be created
        space_id: String,
        /// Type key for the object
        #[arg(short, long)]
        type_key: String,
        /// Object name
        #[arg(short, long)]
        name: Option<String>,
        /// Properties in JSON format (e.g., '{"property":"value"}')
        #[arg(long)]
        properties: Option<String>,
        /// Template ID to use for the object
        #[arg(long)]
        template_id: Option<String>,
    },
    /// Update an existing object in a space
    Update {
        /// Space ID where the object exists
        space_id: String,
        /// Object ID to update
        object_id: String,
        /// New object name
        #[arg(short, long)]
        name: Option<String>,
        /// Properties in JSON format (e.g., '{"property":"value"}')
        #[arg(long)]
        properties: Option<String>,
    },
    /// Delete (archive) an object in a space
    Delete {
        /// Space ID where the object exists
        space_id: String,
        /// Object ID to delete
        object_id: String,
    },
}

pub async fn handle_objects_command(args: ObjectsArgs) -> Result<()> {
    let api_key = crate::config::load_api_key()?
        .ok_or_else(|| anyhow::anyhow!("Not authenticated. Run 'anytype auth login' first."))?;

    let mut client = AnytypeClient::new()?;
    client.set_api_key(api_key);

    match args.command {
        ObjectsCommand::List { space_id, limit } => list_objects(&client, &space_id, limit).await,
        ObjectsCommand::Get {
            space_id,
            object_id,
        } => get_object(&client, &space_id, &object_id).await,
        ObjectsCommand::Create {
            space_id,
            type_key,
            name,
            properties,
            template_id,
        } => create_object(&client, &space_id, &type_key, name, properties, template_id).await,
        ObjectsCommand::Update {
            space_id,
            object_id,
            name,
            properties,
        } => update_object(&client, &space_id, &object_id, name, properties).await,
        ObjectsCommand::Delete {
            space_id,
            object_id,
        } => delete_object(&client, &space_id, &object_id).await,
    }
}

async fn list_objects(client: &AnytypeClient, space_id: &str, limit: Option<u32>) -> Result<()> {
    println!("📦 Fetching objects from space '{space_id}'...");

    let all_objects = client
        .list_all_objects_with_pagination(space_id, limit.map(|l| l as usize))
        .await
        .context("Failed to fetch objects")?;

    if all_objects.is_empty() {
        println!("📭 No objects found in this space.");
        return Ok(());
    }

    println!("✅ Found {} total objects:", all_objects.len());

    // Display pagination summary
    println!("📊 Pagination Summary:");
    if let Some(limit) = limit {
        println!("  • Requested limit: {}", limit);
    }
    println!("  • Objects displayed: {}", all_objects.len());
    println!();

    let all_objects_len = all_objects.len();

    for obj in all_objects {
        println!(
            "  📦 {} ({})",
            obj.name.as_deref().unwrap_or("Unnamed"),
            obj.id
        );
        println!("     🆔 ID: {}", obj.id);

        if let Some(space_id) = &obj.space_id {
            println!("     🏠 Space: {space_id}");
        }

        if let Some(object_type) = &obj.object {
            println!("     📋 Type: {object_type}");
        }

        if !obj.properties.is_null() {
            let prop_count = obj.properties.as_object().map_or(0, |m| m.len());
            println!("     🔑 Properties: {prop_count} properties");
        }

        println!();
    }

    println!(
        "✅ Displayed {} objects from space '{}'",
        all_objects_len, space_id
    );

    Ok(())
}

async fn get_object(client: &AnytypeClient, space_id: &str, object_id: &str) -> Result<()> {
    println!("🔍 Fetching object '{object_id}' from space '{space_id}'...");

    let obj = client
        .get_object(space_id, object_id)
        .await
        .context("Failed to fetch object")?;

    println!("✅ Object found:");
    println!("  📦 Name: {}", obj.name.as_deref().unwrap_or("Unnamed"));
    println!("  🆔 ID: {}", obj.id);

    if let Some(space_id) = &obj.space_id {
        println!("  🏠 Space: {space_id}");
    }

    if let Some(object_type) = &obj.object {
        println!("  📋 Type: {object_type}");
    }

    if !obj.properties.is_null() {
        println!("  🔑 Properties:");
        if let Some(props) = obj.properties.as_object() {
            for (key, value) in props.iter().take(5) {
                println!("    • {}: {}", key, value);
            }
            if props.len() > 5 {
                println!("    ... and {} more properties", props.len() - 5);
            }
        }
    } else {
        println!("  🔑 Properties: None");
    }

    Ok(())
}

async fn create_object(
    client: &AnytypeClient,
    space_id: &str,
    type_key: &str,
    name: Option<String>,
    properties: Option<String>,
    template_id: Option<String>,
) -> Result<()> {
    println!("🏗️ Creating object in space '{space_id}' with type '{type_key}'...");

    let properties_json = if let Some(props_str) = properties {
        serde_json::from_str(&props_str).context("Failed to parse properties JSON")?
    } else {
        json!({})
    };

    let request = CreateObjectRequest {
        type_key: type_key.to_string(),
        name,
        properties: Some(properties_json),
        template_id,
    };

    let response = client
        .create_object(space_id, request)
        .await
        .context("Failed to create object")?;

    println!("✅ Object created successfully!");
    println!(
        "  📦 Name: {}",
        response.object.name.as_deref().unwrap_or("Unnamed")
    );
    println!("  🆔 ID: {}", response.object.id);

    if let Some(space_id) = &response.object.space_id {
        println!("  🏠 Space: {space_id}");
    }

    if let Some(object_type) = &response.object.object {
        println!("  📋 Type: {object_type}");
    }

    if let Some(markdown) = &response.markdown {
        println!("  📝 Content: {} characters", markdown.len());
    }

    Ok(())
}

async fn update_object(
    client: &AnytypeClient,
    space_id: &str,
    object_id: &str,
    name: Option<String>,
    properties: Option<String>,
) -> Result<()> {
    println!("🔄 Updating object '{object_id}' in space '{space_id}'...");

    let properties_json = if let Some(props_str) = properties {
        Some(serde_json::from_str(&props_str).context("Failed to parse properties JSON")?)
    } else {
        None
    };

    let request = UpdateObjectRequest {
        name,
        properties: properties_json,
        markdown: None,
    };

    let response = client
        .update_object(space_id, object_id, request)
        .await
        .context("Failed to update object")?;

    println!("✅ Object updated successfully!");
    println!(
        "  📦 Name: {}",
        response.object.name.as_deref().unwrap_or("Unnamed")
    );
    println!("  🆔 ID: {}", response.object.id);

    if let Some(space_id) = &response.object.space_id {
        println!("  🏠 Space: {space_id}");
    }

    if let Some(object_type) = &response.object.object {
        println!("  📋 Type: {object_type}");
    }

    if let Some(markdown) = &response.markdown {
        println!("  📝 Content: {} characters", markdown.len());
    }

    Ok(())
}

async fn delete_object(client: &AnytypeClient, space_id: &str, object_id: &str) -> Result<()> {
    println!("⚠️ Deleting (archiving) object '{object_id}' in space '{space_id}'...");
    println!("📝 Note: This will mark the object as archived, not permanently delete it.");

    let response = client
        .delete_object(space_id, object_id)
        .await
        .context("Failed to delete object")?;

    println!("✅ Object deleted (archived) successfully!");
    println!(
        "  📦 Name: {}",
        response.object.name.as_deref().unwrap_or("Unnamed")
    );
    println!("  🆔 ID: {}", response.object.id);

    if let Some(space_id) = &response.object.space_id {
        println!("  🏠 Space: {space_id}");
    }

    if let Some(object_type) = &response.object.object {
        println!("  📋 Type: {object_type}");
    }

    Ok(())
}
