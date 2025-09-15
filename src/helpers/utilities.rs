use std::{fs, path::PathBuf, sync::MutexGuard};

use directories::{BaseDirs, UserDirs};
// use floem::reactive::RwSignal;
// use floem::reactive::SignalGet;
// use floem::reactive::SignalUpdate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use stunts_engine::animations::AnimationData;
use stunts_engine::animations::AnimationProperty;
use stunts_engine::animations::BackgroundFill;
use stunts_engine::animations::EasingType;
use stunts_engine::animations::KeyType;
use stunts_engine::animations::KeyframeValue;
use stunts_engine::animations::ObjectType;
use stunts_engine::animations::Sequence;
use stunts_engine::animations::UIKeyframe;
use stunts_engine::editor::wgpu_to_human;
use stunts_engine::editor::PathType;
use stunts_engine::polygon::SavedPoint;
use stunts_engine::polygon::SavedPolygonConfig;
use stunts_engine::polygon::SavedStroke;
use stunts_engine::timelines::SavedTimelineStateConfig;
use stunts_engine::editor::WindowSize;
use uuid::Uuid;

use stunts_engine::saved_state::ProjectData;
use stunts_engine::saved_state::ProjectsDataFile;
use stunts_engine::saved_state::SavedState;

#[cfg(feature = "production")]
pub const API_URL: &str = "https://madebycommon.com";

#[cfg(not(feature = "production"))]
pub const API_URL: &str = "http://localhost:3000";

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthToken {
    pub token: String,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub expiry: Option<chrono::DateTime<chrono::Utc>>,
}

// #[derive(Clone)]
// pub struct AuthState {
//     pub token: Option<AuthToken>,
//     pub is_authenticated: bool,
// }

#[derive(Debug, Clone, Deserialize)]
pub struct Plan {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionDetails {
    pub subscription_status: String,
    pub current_period_end: Option<chrono::DateTime<chrono::Utc>>,
    pub plan: Option<Plan>,
    pub cancel_at_period_end: bool,
}

// Extend AuthState to include subscription details
#[derive(Clone)]
pub struct AuthState {
    pub token: Option<AuthToken>,
    pub is_authenticated: bool,
    pub subscription: Option<SubscriptionDetails>,
}

impl AuthState {
    pub fn can_create_projects(&self) -> bool {
        if !self.is_authenticated {
            return false;
        }

        return true;

        // match &self.subscription {
        //     Some(sub) => matches!(sub.subscription_status.as_str(), "ACTIVE" | "TRIALING"),
        //     None => false,
        // }
    }
}

// Function to fetch subscription details
pub async fn fetch_subscription_details(
    token: &str,
) -> Result<SubscriptionDetails, Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = client
        .get(API_URL.to_owned() + &"/api/subscription/details")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if response.status().is_success() {
        let details = response.json::<SubscriptionDetails>().await?;
        Ok(details)
    } else {
        Err(response.text().await?.into())
    }
}
