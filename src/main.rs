use gui_core::{App, Element};
use gui_core::widgets::*;
use gui_core::widgets::container::{Padding, Background};
use gui_core::widgets::text::text_signal;
use gui_core::widgets::property_inspector::{property_inspector, PropertyGroup, PropertyDefinition, PropertyType};
use gui_core::widgets::dropdown::DropdownOption;
use gui_reactive::Signal;
use vello::peniko::{Color, Gradient, GradientKind, ColorStops, Extend};
use gui_core::widgets::canvas::canvas;
use vello::kurbo::{Circle, RoundedRect};
use vello::{Scene, kurbo::Affine};
use wgpu::{Device, Queue};
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::path::Path;
use stunts_engine::{
    editor::{Viewport, WindowSize, Editor, Point, WindowSizeShader, ObjectProperty},
    capture::{StCapture, get_sources, WindowInfo, MousePosition, SourceData},
    export::exporter::{ExportProgress, Exporter},
    timelines::{SavedTimelineStateConfig, TimelineSequence, TrackType},
};
use stunts_engine::polygon::{
    Polygon, PolygonConfig, SavedPoint, SavedPolygonConfig, SavedStroke, Stroke,
};
use stunts_engine::editor::rgb_to_wgpu;
use stunts_engine::st_image::{SavedStImageConfig, StImage, StImageConfig};
use stunts_engine::st_video::{SavedStVideoConfig, StVideoConfig};
use stunts_engine::text_due::{SavedTextRendererConfig, TextRenderer, TextRendererConfig};
use uuid::Uuid;
use rand::Rng;
use undo::{Edit, Record};
use stunts_engine::{
    animations::{BackgroundFill, Sequence, ObjectType},
};
use winit::event::{ElementState, KeyEvent, Modifiers, MouseButton, MouseScrollDelta};
use winit::dpi::{LogicalSize, PhysicalSize};
use stunts_engine::gpu_resources::GpuResources;
use editor_state::EditorState;
use std::fs;
use stunts_engine::editor::wgpu_to_human;
use keyring::Entry;
use stunts_engine::saved_state::{ProjectData, ProjectsDataFile};
use chrono;
use stunts_engine::saved_state::get_random_coords;
use crate::helpers::utilities::{AuthState, AuthToken};
use anyhow::Result;
use stunts_engine::saved_state::save_saved_state_raw;

mod primary_canvas;
mod pipeline;
mod render_integration;
mod helpers;
mod editor_state;
mod event_handlers;
mod text_properties;
mod theme_sidebar;
mod animation_ideas;

#[derive(Debug, Clone)]
enum Command {
    AddSquarePolygon,
    AddText,
    AddImage { file_path: String },
    AddVideo { file_path: String },
    AddMotion,
    SubmitMotionForm {
        description: String,
        position: String,
        scale: String,
        rotation: String,
        opacity: String,
        delay: String,
        duration: String,
    },
    UpdateTextProperty {
        // text_id: String,
        property_key: String,
        property_value: String,
    },
    TogglePlay,
    ShowCaptureSources,
    StartScreenCapture { hwnd: usize, width: usize, height: usize },
    StopScreenCapture,
    Export {
        progress_tx: tokio_mpsc::UnboundedSender<ExportProgress>,
    },
    // Authentication commands
    SubmitSignIn { email: String, password: String },
    SignOut,
    // CheckAuthentication,
    // Project management commands
    LoadProjects,
    SelectProject { project_id: String },
    CreateProject { name: String },
    CreateSequence { name: String, project_id: String },
    ApplyTheme { theme: [f64; 5] },
}

// Authentication and Project Management structs  
#[derive(Clone, Debug, Serialize, Deserialize)]
struct JwtData {
    token: String,
    expiry: i64, // Unix timestamp
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthResponse {
    #[serde(rename = "jwtData")]
    jwt_data: JwtData,
}

// Intermediate structs to parse the API response format
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(default)]
struct ApiAnimationData {
    pub id: String,
    pub duration: i32, // milliseconds as i32
    pub properties: Vec<ApiAnimationProperty>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(default)]
struct ApiAnimationProperty {
    pub name: String,
    pub keyframes: Vec<ApiKeyframe>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(default)]
struct ApiKeyframe {
    pub time: i32, // milliseconds as i32
    pub value: ApiKeyframeValue,
    pub easing: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ApiKeyframeValue {
    Position { Position: ApiPosition },
    Scale { Scale: i32 },
    Opacity { Opacity: i32 },
    Rotation { Rotation: i32 },
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(default)]
struct ApiPosition {
    pub x: i32,
    pub y: i32,
}

// Default implementations
impl Default for ApiAnimationData {
    fn default() -> Self {
        Self {
            id: String::new(),
            duration: 0,
            properties: Vec::new(),
        }
    }
}

impl Default for ApiAnimationProperty {
    fn default() -> Self {
        Self {
            name: String::new(),
            keyframes: Vec::new(),
        }
    }
}

impl Default for ApiKeyframe {
    fn default() -> Self {
        Self {
            time: 0,
            value: ApiKeyframeValue::Position { Position: ApiPosition::default() },
            easing: "Linear".to_string(),
        }
    }
}

impl Default for ApiPosition {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

// Conversion functions
impl ApiAnimationData {
    fn to_animation_data(self, polygon_id: String, object_type: ObjectType) -> stunts_engine::animations::AnimationData {
        use stunts_engine::animations::{AnimationData, AnimationProperty, UIKeyframe, KeyframeValue, EasingType, ObjectType};
        use stunts_engine::editor::PathType;
        
        // Convert API properties to AnimationProperty
        let properties = self.properties.into_iter().map(|api_prop| {
            let keyframes = api_prop.keyframes.into_iter().map(|api_keyframe| {
                let value = match api_keyframe.value {
                    ApiKeyframeValue::Position { Position: pos } => {
                        KeyframeValue::Position([pos.x, pos.y])
                    }
                    ApiKeyframeValue::Scale { Scale: scale } => {
                        KeyframeValue::Scale(scale)
                    }
                    ApiKeyframeValue::Opacity { Opacity: opacity } => {
                        KeyframeValue::Opacity(opacity)
                    }
                    ApiKeyframeValue::Rotation { Rotation: rotation } => {
                        KeyframeValue::Rotation(rotation)
                    }
                };
                
                let easing = match api_keyframe.easing.as_str() {
                    "EaseIn" => EasingType::EaseIn,
                    "EaseOut" => EasingType::EaseOut,
                    "EaseInOut" => EasingType::EaseInOut,
                    _ => EasingType::Linear,
                };
                
                UIKeyframe {
                    id: uuid::Uuid::new_v4().to_string(),
                    time: Duration::from_millis(api_keyframe.time as u64),
                    value,
                    easing,
                    path_type: PathType::Linear,
                    key_type: stunts_engine::animations::KeyType::Frame,
                }
            }).collect();
            
            AnimationProperty {
                name: api_prop.name,
                property_path: String::new(),
                children: Vec::new(),
                keyframes,
                depth: 0,
            }
        }).collect();
        
        AnimationData {
            id: self.id,
            object_type,
            polygon_id,
            duration: Duration::from_millis(self.duration as u64),
            start_time_ms: 0,
            properties,
            position: [0, 0],
        }
    }
}

fn split_format_string(input: &str) -> Vec<&str> {
    input.split(&['-', 'x'][..]).collect()
}

// Authentication helper functions using keyring for token storage
fn store_auth_token(token: &AuthToken) -> anyhow::Result<()> {
    let entry = Entry::new("stunts-native", "auth_token")?;
    let token_json = serde_json::to_string(token)?;
    entry.set_password(&token_json)?;
    Ok(())
}

fn get_stored_auth_token() -> Option<AuthToken> {
    println!("get_stored_auth_token");
    match Entry::new("stunts-native", "auth_token") {
        Ok(entry) => {
            println!("Entry created successfully");
            match entry.get_password() {
                Ok(token_json) => {
                    println!("Got password from keyring, length: {}", token_json.len());
                    match serde_json::from_str::<AuthToken>(&token_json) {
                        Ok(token) => {
                            // Check if token is expired
                            if let Some(expiry) = token.expiry {
                                let is_valid = expiry > chrono::Utc::now();
                                println!("Token expiry check: expired = {}, expiry = {:?}", !is_valid, expiry);
                                if is_valid {
                                    println!("Returning valid token");
                                    return Some(token);
                                } else {
                                    println!("Token is expired");
                                }
                            } else {
                                println!("Token has no expiry, returning it");
                                return Some(token);
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse token JSON: {}", e);
                            println!("Raw token data: {}", token_json);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to get password from keyring: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create keyring entry: {}", e);
        }
    }
    println!("Returning None - no valid token found");
    None
}

fn clear_stored_auth_token() -> anyhow::Result<()> {
    let entry = Entry::new("stunts-native", "auth_token")?;
    // entry.delete_credential()?; // how to delete a password?
    Ok(())
}

async fn authenticate_user(email: &str, password: &str) -> anyhow::Result<AuthResponse> {
    let client = reqwest::Client::new();
    let auth_data = serde_json::json!({
        "email": email,
        "password": password
    });
    
    let response = client
        .post("http://localhost:3000/api/auth/login")
        .json(&auth_data)
        .send()
        .await?;
    
    if response.status().is_success() {
        let auth_response: AuthResponse = response.json().await?;
        Ok(auth_response)
    } else {
        Err(anyhow::anyhow!("Authentication failed: {}", response.status()))
    }
}

// Load projects locally using utilities.rs functions
fn load_local_projects() -> anyhow::Result<Vec<ProjectData>> {
    let projects_datafile = stunts_engine::saved_state::load_projects_datafile()?;
    Ok(projects_datafile.projects)
}

// Create project locally using utilities.rs functions
fn create_local_project(name: &str) -> anyhow::Result<ProjectData> {
    let saved_state = stunts_engine::saved_state::create_project_state(name.to_string())?;
    
    // Return project data that matches the created project
    Ok(ProjectData {
        project_id: saved_state.id.clone(),
        project_name: name.to_string(),
    })
}

// Helper function to arrange sequences in series automatically
fn arrange_sequences_in_series(sequences: &mut Vec<Sequence>) -> SavedTimelineStateConfig {
    let mut timeline_sequences = Vec::new();
    let mut current_start_time = 0;
    
    for (index, sequence) in sequences.iter().enumerate() {
        timeline_sequences.push(TimelineSequence {
            id: Uuid::new_v4().to_string(),
            sequence_id: sequence.id.clone(),
            start_time_ms: current_start_time,
            track_type: TrackType::Video,
        });
        current_start_time += sequence.duration_ms;
        
        println!("Arranged sequence {} '{}' at time {}ms", index + 1, sequence.name, current_start_time - sequence.duration_ms);
    }
    
    SavedTimelineStateConfig {
        timeline_sequences,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Stunts Native...");

    // Initialize authentication state
    let mut auth_state = AuthState {
        token: None,
        is_authenticated: false,
        subscription: None,
    };

    let auth_state = Signal::new(auth_state.clone());
    
    let mut local_projects: Vec<ProjectData> = Vec::new();
    let mut selected_project: Option<ProjectData> = None;
    
    // Check for stored authentication token
    if let Some(stored_token) = get_stored_auth_token() {
        let new_auth_state = AuthState {
            token: Some(stored_token.clone()),
            is_authenticated: true,
            subscription: None,
        };

        auth_state.set(new_auth_state);
        
        // Try to fetch subscription details to validate token
        match helpers::utilities::fetch_subscription_details(&stored_token.token).await {
            Ok(subscription) => {
                // auth_state.get().is_authenticated = true;
                // auth_state.subscription = Some(subscription);

                println!("get here!!");

                let new_auth_state = AuthState {
                    token: Some(stored_token.clone()),
                    is_authenticated: true,
                    subscription: Some(subscription),
                };

                auth_state.set(new_auth_state);
                
                // Load local projects since user is authenticated
                match load_local_projects() {
                    Ok(projects) => {
                        local_projects = projects;
                        println!("Successfully loaded {} local projects", local_projects.len());
                    }
                    Err(e) => {
                        println!("Failed to load local projects: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to validate stored token: {}", e);
                // Clear invalid token
                let _ = clear_stored_auth_token();
                let new_auth_state = AuthState {
                    token: None,
                    is_authenticated: false,
                    subscription: None,
                };

                auth_state.set(new_auth_state);
            }
        }
    }

    // let saved_state = load_project_state(uuid.clone().to_string())
    //     .expect("Couldn't get Saved State");

    // dummy project
    let project_id = Uuid::new_v4(); // TODO: set with real id for export
    let destination_view = "scene".to_string();
    // let dummy_sequence_id = Uuid::new_v4();

    // let mut dummy_sequences = Vec::new();

    // dummy_sequences.push(Sequence  {
    //     id: current_sequence_id.get().clone(),
    //     name: "Sequence 1".to_string(),
    //     background_fill: Some(BackgroundFill::Color([
    //         wgpu_to_human(0.8) as i32,
    //         wgpu_to_human(0.8) as i32,
    //         wgpu_to_human(0.8) as i32,
    //         255,
    //     ])),
    //     duration_ms: 20000,
    //     active_polygons: Vec::new(),
    //     polygon_motion_paths: Vec::new(),
    //     active_text_items: Vec::new(),
    //     active_image_items: Vec::new(),
    //     active_video_items: Vec::new(),
    // });
    
    // let saved_state = stunts_engine::saved_state::SavedState {
    //     id: project_id.to_string(),
    //     // name: "New Project".to_string(),
    //     sequences: dummy_sequences,
    //     timeline_state: SavedTimelineStateConfig {
    //         timeline_sequences: Vec::new(),
    //     },
    // };

    let window_size = WindowSize {
        width: 1200,
        height: 800,
    };

    let viewport = Arc::new(Mutex::new(Viewport::new(
        window_size.width as f32,
        window_size.height as f32,
    )));

    // let's try with the unified editor.rs
    let mut editor = Editor::new(viewport.clone(), project_id.clone().to_string());

    // Set canvas visibility based on authentication state
    // editor.canvas_hidden = !auth_state.get().is_authenticated && selected_project.is_some();
    editor.canvas_hidden = true;

    // editor.saved_state = Some(saved_state.clone()); // None till loaded
    // editor.project_selected = Some(project_id.clone());
    editor.current_view = destination_view.clone();

    // Create channel for communicating commands from UI to main thread
    let (command_tx, command_rx) = mpsc::channel::<Command>();
    
    // Set up video completion callback for automatic AddVideo command
    editor.st_capture.set_video_completion_callback({
        let tx_for_callback = command_tx.clone();
        
        move |video_path: String| {
            println!("Video encoding completed: {}", video_path);
            let _ = tx_for_callback.send(Command::AddVideo {
                file_path: video_path,
            });
        }
    });

    let editor = Arc::new(Mutex::new(editor));

    // editor_state holds saved data, not active gpu data
    let cloned_editor = Arc::clone(&editor);
    let record = Arc::new(Mutex::new(Record::new()));
    let mut editor_state = editor_state::EditorState::new(cloned_editor, record.clone());
    
    let editor_state = Arc::new(Mutex::new(editor_state));

    // Create channel for API responses
    let (api_response_tx, api_response_rx) = mpsc::channel::<stunts_engine::animations::AnimationData>();
    
    // Create channel for export progress
    let (export_progress_tx, export_progress_rx) = tokio_mpsc::unbounded_channel::<ExportProgress>();

    // Create gradients for button1 states
    let button_normal = Gradient::new_linear((0.0, 0.0), (0.0, 40.0))
        .with_stops([Color::rgb8(75, 75, 80), Color::rgb8(60, 60, 65)]);
    let button_hover = Gradient::new_linear((0.0, 0.0), (0.0, 40.0))
        .with_stops([Color::rgb8(85, 85, 90), Color::rgb8(70, 70, 75)]);
    let button_pressed = Gradient::new_linear((0.0, 0.0), (0.0, 40.0))
        .with_stops([Color::rgb8(50, 50, 55), Color::rgb8(65, 65, 70)]);

    let display_motion_form = Signal::new(false);
    let display_motion_loading = Signal::new(false);
    
    // Screen capture state
    let capture_sources_visible = Signal::new(false);
    let available_capture_sources = Signal::new(Vec::<DropdownOption>::new());
    let is_recording = Signal::new(false);
    
    // Export state
    let is_exporting = Signal::new(false);
    let export_progress = Signal::new(0.0f32);
    let export_status = Signal::new("Ready to export".to_string());
    
    // Sidebar state for property editing
    let sidebar_visible = Signal::new(false);
    let sidebar_width = 300.0;
    let text_properties_visible = Signal::new(false);
    let themes_sidebar_visible = Signal::new(false);
    
    // Authentication state signals
    let local_projects_signal = Signal::new(local_projects.clone());
    let selected_project_signal = Signal::new(selected_project.clone());
    let current_sequence_id = Signal::new(String::new());
    let show_editor = Signal::new(auth_state.get().is_authenticated && selected_project.is_some());
    let show_auth_form = Signal::new(!auth_state.get().is_authenticated);
    let show_project_list = Signal::new(auth_state.get().is_authenticated && selected_project.is_none());
    let show_project_creation = Signal::new(false);
    let auth_loading = Signal::new(false);
    
    // Auth form fields
    let email_text = Signal::new("".to_string());
    let password_text = Signal::new("".to_string());
    let project_name_text = Signal::new("".to_string());

    let motion_text = Signal::new("Motion Direction".to_string());
    let description_text = Signal::new("".to_string());
    let position_text = Signal::new("".to_string());
    let scale_text = Signal::new("".to_string());
    let opacity_text = Signal::new("".to_string());
    let rotation_text = Signal::new("".to_string());
    let duration_text = Signal::new("".to_string());
    let delay_text = Signal::new("".to_string());

    let motion_form = container()
        .with_size(400.0, 450.0)
        .with_background_color(Color::rgba8(255, 200, 150, 200))
        .with_border_radius(12.0)
        .with_padding(Padding::all(15.0))
        .with_shadow(4.0, 4.0, 8.0, Color::rgba8(0, 0, 0, 100))
        .with_display_signal(display_motion_form.clone())
        .with_child(
            // Element::new_widget(Box::new(
            column()
                .with_size(400.0, 450.0)
                .with_main_axis_alignment(MainAxisAlignment::Start)
                .with_cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(Element::new_widget(Box::new(
                    text_signal(motion_text.clone())
                        .with_font_size(16.0)
                        .with_color(Color::rgba8(100, 50, 0, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Object Description:")
                        .with_font_size(12.0)
                        .with_color(Color::rgba8(80, 40, 0, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(350.0)
                        .with_height(30.0)
                        .with_placeholder("ex. a basketball")
                        .with_signal(description_text.clone())
                        .on_change({
                            let description_text = description_text.clone();

                            move |text| {
                                // description_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Position:")
                        .with_font_size(12.0)
                        .with_color(Color::rgba8(80, 40, 0, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(350.0)
                        .with_height(30.0)
                        .with_placeholder("ex. bounces up and down")
                        .with_signal(position_text.clone())
                        .on_change({
                            let position_text = position_text.clone();

                            move |text| {
                                // position_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Scale:")
                        .with_font_size(12.0)
                        .with_color(Color::rgba8(80, 40, 0, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(350.0)
                        .with_height(30.0)
                        .with_placeholder("ex. scales to fullscreen")
                        .with_signal(scale_text.clone())
                        .on_change({
                            let scale_text = scale_text.clone();

                            move |text| {
                                // scale_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Opacity:")
                        .with_font_size(12.0)
                        .with_color(Color::rgba8(80, 40, 0, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(350.0)
                        .with_height(30.0)
                        .with_placeholder("ex. fades in")
                        .with_signal(opacity_text.clone())
                        .on_change({
                            let opacity_text = opacity_text.clone();

                            move |text| {
                                // opacity_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Rotation:")
                        .with_font_size(12.0)
                        .with_color(Color::rgba8(80, 40, 0, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(350.0)
                        .with_height(30.0)
                        .with_placeholder("ex. spins 360 degrees")
                        .with_signal(rotation_text.clone())
                        .on_change({
                            let rotation_text = rotation_text.clone();

                            move |text| {
                                // rotation_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Duration:")
                        .with_font_size(12.0)
                        .with_color(Color::rgba8(80, 40, 0, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(350.0)
                        .with_height(30.0)
                        .with_placeholder("ex. Slow, 5 seconds, 1500ms")
                        .with_signal(duration_text.clone())
                        .on_change({
                            let duration_text = duration_text.clone();

                            move |text| {
                                // rotation_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Delay:")
                        .with_font_size(12.0)
                        .with_color(Color::rgba8(80, 40, 0, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(350.0)
                        .with_height(30.0)
                        .with_placeholder("ex. Momentary, 2 seconds, 1500ms")
                        .with_signal(delay_text.clone())
                        .on_change({
                            let delay_text = delay_text.clone();

                            move |text| {
                                // rotation_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    button("Try Random Animation")
                        .with_font_size(12.0)
                        .with_width(150.0)
                        .with_height(30.0)
                        .with_backgrounds(
                            Background::Gradient(button_normal.clone()),
                            Background::Gradient(button_hover.clone()),
                            Background::Gradient(button_pressed.clone())
                        )
                        .on_click({
                            let description_text = description_text.clone();
                            let position_text = position_text.clone();
                            let scale_text = scale_text.clone();
                            let opacity_text = opacity_text.clone();
                            let rotation_text = rotation_text.clone();

                            move || {
                                let ideas = crate::animation_ideas::get_animation_ideas();
                                if !ideas.is_empty() {
                                    let random_index = rand::random::<usize>() % ideas.len();
                                    let random_idea = &ideas[random_index];

                                    println!("set descrip {:?}", random_idea.object_description.clone());
                                    
                                    description_text.set(random_idea.object_description.clone());
                                    position_text.set(random_idea.position_description.clone());
                                    scale_text.set(random_idea.scale_description.clone());
                                    opacity_text.set(random_idea.opacity_description.clone());
                                    rotation_text.set(random_idea.rotation_description.clone());
                                }
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    button("Confirm")
                        .with_font_size(12.0)
                        .with_width(100.0)
                        .with_height(30.0)
                        .with_backgrounds(
                            Background::Gradient(button_normal.clone()),
                            Background::Gradient(button_hover.clone()),
                            Background::Gradient(button_pressed.clone())
                        )
                        .on_click({
                            let display_motion_form = display_motion_form.clone();
                            let description = description_text.clone();
                            let position = position_text.clone();
                            let scale = scale_text.clone();
                            let opacity = opacity_text.clone();
                            let rotation = rotation_text.clone();
                            let duration = duration_text.clone();
                            let delay = delay_text.clone();
                            let editor_for_api = editor.clone();
                            let tx = command_tx.clone();

                            move || {
                                // Send command to be processed in async context
                                tx.send(Command::SubmitMotionForm {
                                    description: description.get(),
                                    position: position.get(),
                                    scale: scale.get(),
                                    opacity: opacity.get(),
                                    rotation: rotation.get(),
                                    duration: duration.get(),
                                    delay: delay.get(),
                                });
                            }
                        })
                )))
                .into_container_element()
        // ))
    );
    
    // Authentication Form
    let auth_form = container()
        .with_size(400.0, 300.0)
        .with_background_color(Color::rgba8(50, 50, 60, 240))
        .with_border_radius(12.0)
        .with_padding(Padding::all(20.0))
        .with_shadow(4.0, 4.0, 8.0, Color::rgba8(0, 0, 0, 150))
        .with_display_signal(show_auth_form.clone())
        .absolute()
        .with_position(50.0, 50.0)
        .with_child(
            column()
                .with_size(400.0, 300.0)
                .with_main_axis_alignment(MainAxisAlignment::Start)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(Element::new_widget(Box::new(
                    text("Sign In")
                        .with_font_size(24.0)
                        .with_color(Color::rgba8(255, 255, 255, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    text("")
                        .with_font_size(12.0) // Spacer
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Email:")
                        .with_font_size(14.0)
                        .with_color(Color::rgba8(200, 200, 200, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(320.0)
                        .with_height(35.0)
                        .with_placeholder("Enter your email")
                        .on_change({
                            let email_text = email_text.clone();
                            move |text| {
                                email_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Password:")
                        .with_font_size(14.0)
                        .with_color(Color::rgba8(200, 200, 200, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(320.0)
                        .with_height(35.0)
                        .with_placeholder("Enter your password")
                        .on_change({
                            let password_text = password_text.clone();
                            move |text| {
                                password_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("")
                        .with_font_size(12.0) // Spacer
                )))
                .with_child(Element::new_widget(Box::new(
                    button("Sign In")
                        .with_font_size(14.0)
                        .with_width(120.0)
                        .with_height(40.0)
                        .with_backgrounds(
                            Background::Gradient(button_normal.clone()),
                            Background::Gradient(button_hover.clone()),
                            Background::Gradient(button_pressed.clone())
                        )
                        .on_click({
                            let email = email_text.clone();
                            let password = password_text.clone();
                            let tx = command_tx.clone();
                            move || {
                                tx.send(Command::SubmitSignIn {
                                    email: email.get(),
                                    password: password.get(),
                                });
                            }
                        })
                )))
                .into_container_element()
        );

        let form_header = column()
                .with_size(800.0, 120.0)
                .with_main_axis_alignment(MainAxisAlignment::Start)
                .with_cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(Element::new_widget(Box::new(
                    text("Select Project")
                        .with_font_size(24.0)
                        .with_color(Color::rgba8(255, 255, 255, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Your Projects:")
                        .with_font_size(16.0)
                        .with_color(Color::rgba8(200, 200, 200, 255))
                )));
    
    // Project Selection Form
    let project_selection_form = container()
        .with_size(800.0, 700.0)
        .with_background_color(Color::rgba8(50, 50, 60, 240))
        .with_border_radius(12.0)
        .with_padding(Padding::all(20.0))
        .with_shadow(4.0, 4.0, 8.0, Color::rgba8(0, 0, 0, 150))
        .with_display_signal(show_project_list.clone())
        .absolute()
        .with_position(50.0, 50.0)
        .with_child(
            column()
                .with_size(800.0, 300.0)
                .with_main_axis_alignment(MainAxisAlignment::Start)
                .with_cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(form_header.into_container_element())
                .with_child(row()
                        .with_size(600.0, 40.0)
                        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                        .with_cross_axis_alignment(CrossAxisAlignment::Center)
                        .with_child(Element::new_widget(Box::new(
                            button("Create Project")
                                .with_font_size(14.0)
                                .with_width(150.0)
                                .with_height(30.0)
                                .with_backgrounds(
                                    Background::Gradient(button_normal.clone()),
                                    Background::Gradient(button_hover.clone()),
                                    Background::Gradient(button_pressed.clone())
                                )
                                .on_click({
                                    let show_project_creation = show_project_creation.clone();
                                    let show_project_list = show_project_list.clone();
                                    move || {
                                        show_project_list.set(false);
                                        show_project_creation.set(true);
                                    }
                                })
                        )))
                        .with_child(Element::new_widget(Box::new(
                            button("Sign Out")
                                .with_font_size(12.0)
                                .with_width(80.0)
                                .with_height(30.0)
                                .with_backgrounds(
                                    Background::Gradient(button_normal.clone()),
                                    Background::Gradient(button_hover.clone()),
                                    Background::Gradient(button_pressed.clone())
                                )
                                .on_click({
                                    let tx = command_tx.clone();
                                    move || {
                                        tx.send(Command::SignOut);
                                    }
                                })
                        ))).into_container_element()
                )
                .with_child(
                    column()
                        .with_size(800.0, 300.0)
                        .with_main_axis_alignment(MainAxisAlignment::Start)
                        .with_cross_axis_alignment(CrossAxisAlignment::Start)
                        .with_reactive_children(local_projects_signal.clone(), {
                            let command_tx = command_tx.clone();

                            move |project_list| {
                            let mut children = Vec::new();

                            for project in project_list {
                                children.push(Element::new_widget(Box::new(
                                    button(&project.project_name)
                                        .with_size(260.0, 26.0)
                                        // .with_background_color(Color::rgba8(70, 70, 80, 255))
                                        .with_font_size(14.0)
                                        .on_click({                               
                                            let tx = command_tx.clone();
                                            let project_id = project.project_id.clone();
                                            
                                            move || {
                                                // Handle project selection    
                                                tx.send(Command::SelectProject { project_id: project_id.clone() });
                                            }
                                        })
                                )));

                                // children.push(Element::new_widget(Box::new(
                                //     text("").with_font_size(4.0) // spacer
                                // )));
                            }

                            children
                        }}).into_container_element()
                )
                .into_container_element()
        );
    
    // Project Creation Form
    let project_creation_form = container()
        .with_size(800.0, 600.0)
        .with_background_color(Color::rgba8(50, 50, 60, 240))
        .with_border_radius(12.0)
        .with_padding(Padding::all(20.0))
        .with_shadow(4.0, 4.0, 8.0, Color::rgba8(0, 0, 0, 150))
        .with_display_signal(show_project_creation.clone())
        .absolute()
        .with_position(50.0, 50.0)
        .with_child(
            column()
                .with_size(800.0, 600.0)
                .with_main_axis_alignment(MainAxisAlignment::Start)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(Element::new_widget(Box::new(
                    text("Create Project")
                        .with_font_size(24.0)
                        .with_color(Color::rgba8(255, 255, 255, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    text("")
                        .with_font_size(12.0) // Spacer
                )))
                .with_child(Element::new_widget(Box::new(
                    text("Project Name:")
                        .with_font_size(14.0)
                        .with_color(Color::rgba8(200, 200, 200, 255))
                )))
                .with_child(Element::new_widget(Box::new(
                    input()
                        .with_width(320.0)
                        .with_height(35.0)
                        .with_placeholder("Enter project name")
                        .on_change({
                            let project_name_text = project_name_text.clone();
                            move |text| {
                                project_name_text.set(text.to_string());
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    text("")
                        .with_font_size(8.0) // Spacer
                )))
                .with_child(Element::new_widget(Box::new(
                    row()
                        .with_size(320.0, 40.0)
                        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                        .with_cross_axis_alignment(CrossAxisAlignment::Center)
                        .with_child(Element::new_widget(Box::new(
                            button("Cancel")
                                .with_font_size(14.0)
                                .with_width(100.0)
                                .with_height(35.0)
                                .with_backgrounds(
                                    Background::Gradient(button_normal.clone()),
                                    Background::Gradient(button_hover.clone()),
                                    Background::Gradient(button_pressed.clone())
                                )
                                .on_click({
                                    let show_project_creation = show_project_creation.clone();
                                    let show_project_list = show_project_list.clone();
                                    move || {
                                        show_project_creation.set(false);
                                        show_project_list.set(true);
                                    }
                                })
                        )))
                        .with_child(Element::new_widget(Box::new(
                            button("Create Project")
                                .with_font_size(14.0)
                                .with_width(300.0)
                                .with_height(35.0)
                                .with_backgrounds(
                                    Background::Gradient(button_normal.clone()),
                                    Background::Gradient(button_hover.clone()),
                                    Background::Gradient(button_pressed.clone())
                                )
                                .on_click({
                                    let project_name = project_name_text.clone();
                                    let tx = command_tx.clone();
                                    move || {
                                        tx.send(Command::CreateProject {
                                            name: project_name.get(),
                                        });
                                    }
                                })
                        )))
                )))
                .into_container_element()
        );
    
    let button_square = button("Add Square")
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            move || {                
                tx.send(Command::AddSquarePolygon);
            }
        });

    let button_text = button("Add Text")
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            move || {                
                tx.send(Command::AddText);
            }
        });

    let button_image = button("Add Image")
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            move || {
                // Spawn a task to handle the file dialog
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    if let Some(file_path) = FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"])
                        .pick_file()
                    {
                        if let Some(path_str) = file_path.to_str() {
                            let _ = tx_clone.send(Command::AddImage { 
                                file_path: path_str.to_string() 
                            });
                        }
                    }
                });
            }
        });

    let button_video = button("Add Video")
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            move || {
                // Spawn a task to handle the file dialog
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    if let Some(file_path) = FileDialog::new()
                        .add_filter("Videos", &["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm", "m4v"])
                        .pick_file()
                    {
                        if let Some(path_str) = file_path.to_str() {
                            let _ = tx_clone.send(Command::AddVideo { 
                                file_path: path_str.to_string() 
                            });
                        }
                    }
                });
            }
        });

    // let capture_button_text = Signal::new("Screen Capture".to_string());

    // let capture_button_text = if is_recording.get() {
    //     "Stop Recording"
    // } else {
    //     "Screen Capture"
    // }.to_string();
    let capture_button_text = Signal::new("Screen Capture".to_string());
    let button_capture = button_signal(capture_button_text.clone())
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            let is_recording = is_recording.clone();
            let capture_sources_visible = capture_sources_visible.clone();
            move || {
                if is_recording.get() {
                    tx.send(Command::StopScreenCapture);
                } else {
                    tx.send(Command::ShowCaptureSources);
                }
            }
        });
    
    // turns on a mode in the editor so the user can draw arrows by clicking and dragging or dots by just clicking
    let motion_button = button("Add Motion")
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            move || {
                tx.send(Command::AddMotion);
            }
        });

    // optional button for regenerate each object according to its arrow (maybe too much at once? could batch it)
    // let button3 = button("Regenerate All")
    //     .with_font_size(10.0)
    //     .with_width(90.0)
    //     .with_height(20.0)
    //     .with_backgrounds(
    //         Background::Gradient(button_normal.clone()),
    //         Background::Gradient(button_hover.clone()),
    //         Background::Gradient(button_pressed.clone())
    //     )
    //     .on_click({
    //         let tx = command_tx.clone();
    //         move || {
    //             tx.send(Command::AddMotion);
    //         }
    //     });

    // export the video - create reactive button text
    let export_button_text = Signal::new("Export".to_string());
    let export_button = button_signal(export_button_text.clone())
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            let export_progress_tx = export_progress_tx.clone();
            let is_exporting = is_exporting.clone();
            move || {
                if !is_exporting.get() {
                    tx.send(Command::Export {
                        progress_tx: export_progress_tx.clone(),
                    });
                }
            }
        });

    // Play / pause the video
    let button5 = button("Play / Pause")
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let tx = command_tx.clone();
            move || {
                tx.send(Command::TogglePlay);
            }
        });

    // Toggle sidebar for property editing
    let button_properties = button("Properties")
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let sidebar_visible = sidebar_visible.clone();
            let text_properties_visible = text_properties_visible.clone();

            move || {
                sidebar_visible.set(!sidebar_visible.get());
                text_properties_visible.set(!text_properties_visible.get());
            }
        });

    // Toggle sidebar for themes
    let button_themes = button("Themes")
        .with_font_size(10.0)
        .with_width(90.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let sidebar_visible = sidebar_visible.clone();
            let themes_sidebar_visible = themes_sidebar_visible.clone();

            move || {
                sidebar_visible.set(!sidebar_visible.get());
                themes_sidebar_visible.set(!themes_sidebar_visible.get());
            }
        });

        // Screen capture sources dropdown
    let capture_sources_dropdown = container()
        .absolute() // Position absolutely
        .with_position(450.0, 5.0) // Position over canvas area
        .with_size(350.0, 40.0)
        .with_background_color(Color::rgba8(255, 255, 255, 240))
        .with_border_radius(8.0)
        // .with_padding(Padding::all(15.0))
        // .with_shadow(4.0, 4.0, 8.0, Color::rgba8(0, 0, 0, 100))
        .with_display_signal(capture_sources_visible.clone())
        .with_child(
            row()
                // .with_size(320.0, 370.0)
                .with_main_axis_alignment(MainAxisAlignment::Start)
                .with_cross_axis_alignment(CrossAxisAlignment::Start)
                // .with_child(Element::new_widget(Box::new(
                //     text("Select Screen or Window to Capture:")
                //         .with_font_size(14.0)
                //         .with_color(Color::rgba8(60, 60, 60, 255))
                // )))
                .with_child(Element::new_widget(Box::new(
                    // // Scrollable list area for capture sources - will be populated dynamically
                    // container()
                    //     .with_size(310.0, 280.0)
                    //     .with_background_color(Color::rgba8(248, 248, 248, 255))
                    //     .with_border_radius(4.0)
                    //     .with_padding(Padding::all(5.0))
                    //     .with_child(
                    //         column()
                    //             .with_size(300.0, 270.0)
                    //             .with_main_axis_alignment(MainAxisAlignment::Start)
                    //             .with_cross_axis_alignment(CrossAxisAlignment::Start)
                    //             // Source buttons will be added here dynamically via command processing
                    //     )
                    // Replace the container section (lines 650-661) with:
                    dropdown()
                        .with_size(150.0, 20.0)
                        .with_font_size(12.0)
                        .with_placeholder("Select a source...")
                        // .with_options({
                        //     let sources = available_capture_sources.get();
                        //     sources.into_iter().map(|source| {
                        //         DropdownOption {
                        //             label: format!("{} ({}x{})", source.title, source.rect.width, source.rect.height),
                        //             value: source.hwnd.to_string(),
                        //         }
                        //     }).collect()
                        // })
                        .with_options_signal(available_capture_sources.clone())
                        .on_selection_changed({
                            let tx = command_tx.clone();
                            let capture_sources_visible = capture_sources_visible.clone();
                            move |selected_value_code: String| {
                                let parts: Vec<&str> = split_format_string(&selected_value_code);
                                if let Ok(hwnd) = parts[0].parse::<usize>() {
                                    if let Ok(width) = parts[1].parse::<usize>() {
                                        if let Ok(height) = parts[2].parse::<usize>() {
                                            println!("Selected capture source HWND: {}", hwnd);
                                            capture_sources_visible.set(false);
                                            let _ = tx.send(Command::StartScreenCapture { hwnd, width, height });
                                        }
                                    }
                                }
                            }
                        })
                )))
                .with_child(Element::new_widget(Box::new(
                    row()
                        .with_size(160.0, 30.0)
                        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                        .with_cross_axis_alignment(CrossAxisAlignment::Center)
                        .with_child(Element::new_widget(Box::new(
                            button("Cancel")
                                .with_font_size(12.0)
                                .with_width(80.0)
                                .with_height(25.0)
                                .with_backgrounds(
                                    Background::Gradient(button_normal.clone()),
                                    Background::Gradient(button_hover.clone()),
                                    Background::Gradient(button_pressed.clone())
                                )
                                .on_click({
                                    let capture_sources_visible = capture_sources_visible.clone();
                                    move || {
                                        capture_sources_visible.set(false);
                                    }
                                })
                        )))
                        .with_child(Element::new_widget(Box::new(
                            button("Refresh")
                                .with_font_size(12.0)
                                .with_width(80.0)
                                .with_height(25.0)
                                .with_backgrounds(
                                    Background::Gradient(button_normal.clone()),
                                    Background::Gradient(button_hover.clone()),
                                    Background::Gradient(button_pressed.clone())
                                )
                                .on_click({
                                    let tx = command_tx.clone();
                                    move || {
                                        tx.send(Command::ShowCaptureSources);
                                    }
                                })
                        )))
                )))
                .into_container_element()
            );    

    let top_tools = row()
        .with_size(1200.0, 50.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        // .with_child(Element::new_widget(Box::new(motion_button)))
        // .with_child(Element::new_widget(Box::new(button_square)))
        // .with_child(Element::new_widget(Box::new(button_text)))
        // .with_child(Element::new_widget(Box::new(button_image)))
        // .with_child(Element::new_widget(Box::new(button_video)))
        // .with_child(Element::new_widget(Box::new(button_capture)))
        // .with_child(capture_sources_dropdown.into_container_element())
        // .with_child(Element::new_widget(Box::new(button3)))
        .with_child(Element::new_widget(Box::new(button_properties)))
        .with_child(Element::new_widget(Box::new(button_themes)))
        .with_child(Element::new_widget(Box::new(export_button)))
        .with_child(Element::new_widget(Box::new(
            text_signal(export_status.clone())
                .with_font_size(10.0)
                .with_color(Color::rgba8(200, 200, 200, 255))
        )));

    let bottom_tools = row()
        .with_size(1200.0, 50.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Element::new_widget(Box::new(motion_button)))
        .with_child(Element::new_widget(Box::new(button_square)))
        .with_child(Element::new_widget(Box::new(button_text)))
        .with_child(Element::new_widget(Box::new(button_image)))
        .with_child(Element::new_widget(Box::new(button_video)))
        .with_child(Element::new_widget(Box::new(button_capture)))
        .with_child(capture_sources_dropdown.into_container_element());
        // .with_child(Element::new_widget(Box::new(button3)))
        // .with_child(Element::new_widget(Box::new(export_button)))
        // .with_child(Element::new_widget(Box::new(button_properties)))
        // .with_child(Element::new_widget(Box::new(button_themes)))
        // .with_child(Element::new_widget(Box::new(
        //     text_signal(export_status.clone())
        //         .with_font_size(10.0)
        //         .with_color(Color::rgba8(200, 200, 200, 255))
        // ))
    // );

    // let right_tools = row()
    //     .with_size(400.0, 50.0)
    //     .with_main_axis_alignment(MainAxisAlignment::Start) // horiz
    //     .with_cross_axis_alignment(CrossAxisAlignment::Start) // vert
    //     .with_child(Element::new_widget(Box::new(button3)))
    //     .with_child(Element::new_widget(Box::new(button4)));

    let toolkit_inner = column()
            .with_size(1200.0, 50.0)
            .with_child(top_tools.into_container_element())
            .with_child(bottom_tools.into_container_element());

    let toolkit = container()
        .with_size(1200.0, 50.0)
        .absolute() // Position absolutely - won't affect layout flow
        .with_position(20.0, 20.0) // Position at specific coordinates
        .with_child(toolkit_inner.into_container_element());
        // .with_child(right_tools.into_container_element());

    let video_ctrls = row()
        .with_size(1200.0, 50.0)
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(button5)));
    
    let video_ctrls_container = container()
        .absolute() // Position absolutely - won't affect layout flow
        .with_position(20.0, 680.0) // Position at specific coordinates
        .with_size(1200.0, 50.0)
        .with_child(video_ctrls.into_container_element());

    // Create text properties widget using the new manual implementation
    let text_properties_widget = text_properties::create_text_properties_panel(
        command_tx.clone(),
        button_normal.clone(),
        button_hover.clone(),
        button_pressed.clone(),
        sidebar_width,
    );

    // Create themes sidebar widget  
    let themes_sidebar_widget = theme_sidebar::create_themes_sidebar_panel(
        command_tx.clone(),
        sidebar_width,
    );

    let text_properties_container = container()
                .with_display_signal(text_properties_visible.clone())
                .with_child(text_properties_widget);

    let themes_sidebar_container = container()
                .with_display_signal(themes_sidebar_visible.clone())
                .with_child(themes_sidebar_widget);

    let sidebar_inner = column()
        .with_size(sidebar_width, 750.0)
        .with_child(text_properties_container.into_container_element())
        .with_child(themes_sidebar_container.into_container_element());

    let property_sidebar = container()
        .absolute() // Position absolutely - won't affect layout flow
        .with_position(20.0, 50.0) // Position at specific coordinates
        .with_size(sidebar_width, 750.0)
        .with_background_color(Color::rgba8(45, 45, 50, 255))
        .with_padding(Padding::only(25.0, 15.0, 25.0, 15.0))
        .with_display_signal(sidebar_visible.clone())
        .with_child(sidebar_inner.into_container_element());

    let container_gradient = Gradient::new_radial((0.0, 0.0), 450.0)
        .with_stops([Color::rgb8(90, 90, 95), Color::rgb8(45, 45, 50)]);

    let main_column = column()
        .with_size(1200.0, 800.0) 
        // .with_radial_gradient(container_gradient)
        // .with_padding(Padding::all(20.0))
        // .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        .with_child(toolkit.into_container_element())
        // .with_child(primary_canvas::create_render_placeholder()?)
        .with_child(video_ctrls_container.into_container_element());

    // Create main content area with sidebar
    let main_content = row()
        .with_size(1200.0, 800.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(property_sidebar.into_container_element())
        .with_child(main_column.into_container_element());

    let editor_container = container()
        .with_size(1200.0, 800.0) 
        .with_display_signal(show_editor.clone())
        // .with_radial_gradient(container_gradient)
        // .with_padding(Padding::all(20.0))
        // .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        .with_child(main_content.into_container_element());
    
    let main_column = column()
        .with_size(1200.0, 800.0) 
        // .with_radial_gradient(container_gradient)
        // .with_padding(Padding::all(20.0))
        // .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        .with_child(project_creation_form.into_container_element())
        .with_child(project_selection_form.into_container_element())
        .with_child(auth_form.into_container_element()) 
        .with_child(motion_form.into_container_element())
        .with_child(editor_container.into_container_element());

    let main_container = container()
        .with_size(1200.0, 800.0) 
        .with_radial_gradient(container_gradient)
        .with_padding(Padding::all(20.0))
        .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        .with_child(main_column.into_container_element());
    
    let root = main_container.into_container_element();

    println!("UI Tree Built! Launching...");
        
    // Start the application with the UI tree and event handlers
    let app = App::new()
        .with_title("Stunts".to_string())?
        .with_inner_size([window_size.width as i32, window_size.height as i32])?
        .with_root(root)?
        .with_cursor_moved({
            let editor = editor.clone();
            let viewport = viewport.clone();
            move |position_x: f64, position_y: f64, log_pos_x: f64, log_pos_y: f64| {
                if let Some(handler) = event_handlers::handle_cursor_moved(
                    editor.clone(),
                    viewport.clone(),
                ) {
                    handler(position_x, position_y, log_pos_x, log_pos_y);
                }
            }
        })
        .with_mouse_input({
            let editor = editor.clone();
            let editor_state = editor_state.clone();
            let viewport = viewport.clone();
            let record = record.clone();
            move |button, state| {
                if let Some(handler) = event_handlers::handle_mouse_input(
                    editor_state.clone(),
                    editor.clone(),
                    viewport.clone(),
                    record.clone(),
                ) {
                    handler(button, state);
                }
            }
        })
        .with_window_resize({
            let editor = editor.clone();
            let viewport = viewport.clone();
            move |size, logical_size| {
                if let Some(mut handler) = event_handlers::handle_window_resize(
                    editor.clone(),
                    viewport.clone(),
                ) {
                    handler(size, logical_size);
                }
            }
        })
        .with_mouse_wheel({
            let editor = editor.clone();
            let viewport = viewport.clone();
            move |delta| {
                if let Some(mut handler) = event_handlers::handle_mouse_wheel(
                    editor.clone(),
                    viewport.clone(),
                ) {
                    handler(delta);
                }
            }
        })
        .with_modifiers_changed({
            let editor_state = editor_state.clone();
            let viewport = viewport.clone();
            move |modifiers| {
                if let Some(mut handler) = event_handlers::handle_modifiers_changed(
                    editor_state.clone(),
                    viewport.clone(),
                ) {
                    handler(modifiers);
                }
            }
        })
        .with_keyboard_input({
            let editor_state = editor_state.clone();
            let viewport = viewport.clone();
            move |event| {
                if let Some(mut handler) = event_handlers::handle_keyboard_input(
                    editor_state.clone(),
                    viewport.clone(),
                ) {
                    handler(event);
                }
            }
        });

    // Use the new run_with_editor_state method that avoids Send + Sync constraints
    app.run_with_editor_state(
        editor.clone(),
        {
            let editor_for_init = editor.clone();
            let viewport_for_init = viewport.clone();
            move |device, queue| {
                let gpu_resources = std::sync::Arc::new(stunts_engine::gpu_resources::GpuResources::from_commonui(device, queue));
                pipeline::init_pipeline(viewport_for_init, editor_for_init, gpu_resources.clone());
                // render_integration::set_gpu_resources(gpu_resources);
            }
        },
        {
            let editor_for_render = editor.clone();
            let state_for_render = editor_state.clone();
            let command_rx_for_render = Arc::new(Mutex::new(command_rx));
            let api_response_rx_for_render = Arc::new(Mutex::new(api_response_rx));
            let api_response_tx_for_render = api_response_tx.clone();
            let export_progress_rx_for_render = Arc::new(Mutex::new(export_progress_rx));
            let engine_handle_cache: RefCell<Option<render_integration::EngineHandle>> = RefCell::new(None);
            
            Arc::new(move |device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, external_resources: &[vello::ExternalResource<'_>], view: &wgpu::TextureView| -> Result<(), vello::Error> {
                // Check if motion arrow was just placed and show form
                if let Ok(mut editor) = editor_for_render.try_lock() {
                    if editor.motion_arrow_just_placed {
                        editor.motion_arrow_just_placed = false;
                        display_motion_form.set(true);
                    }
                }
                
                // Process API responses
                // uses std, not tokio
                if let Ok(rx) = api_response_rx_for_render.try_lock() {
                    while let Ok(animation_data) = rx.try_recv() {
                        if let Ok(mut editor) = editor_for_render.try_lock() {
                            if let Ok(mut editor_state) = state_for_render.try_lock() { 
                                let sequence_data = editor.current_sequence_data.clone();
                                let last_motion_arrow_object_id = editor.last_motion_arrow_object_id.to_string();
                                let last_motion_arrow_object_type = editor.last_motion_arrow_object_type.clone();
                               
                                if let Some(ref mut saved_state) = editor.saved_state {
                                    // Clean up data
                                    let mut final_animation = animation_data.clone();
                                    final_animation.id = Uuid::new_v4().to_string();
                                    // final_animation.object_type = ObjectType::Polygon;
                                    final_animation.polygon_id = last_motion_arrow_object_id;
                                    final_animation.start_time_ms = 0;
                                    final_animation.position = [0, 0];

                                    if last_motion_arrow_object_type == ObjectType::VideoItem {
                                        let zoom_prop = editor_state.save_default_zoom();

                                        final_animation.properties.push(zoom_prop);
                                    }

                                    println!("final_animation: {:?}", final_animation);


                                    // Find the current sequence and add/overwrite the animation data
                                    if let Some(current_seq_data) = &sequence_data {
                                        for sequence in &mut saved_state.sequences {
                                            if sequence.id == current_seq_data.id {
                                                // Remove any existing motion paths for this polygon_id
                                                sequence.polygon_motion_paths.retain(|path| 
                                                    path.polygon_id != final_animation.polygon_id
                                                );
                                                
                                                // Add the new motion path
                                                sequence.polygon_motion_paths.push(final_animation.clone());
                                                break;
                                            }
                                        }
                                        
                                        // Update the editor's current_sequence_data
                                        let mut updated_sequence = current_seq_data.clone();
                                        
                                        // Remove existing motion paths for this polygon_id from the editor's sequence
                                        updated_sequence.polygon_motion_paths.retain(|path| 
                                            path.polygon_id != final_animation.polygon_id
                                        );
                                        
                                        // Add the new motion path
                                        updated_sequence.polygon_motion_paths.push(final_animation);
                                        editor.current_sequence_data = Some(updated_sequence.clone());
                                        
                                        // Call update_motion_paths to refresh the editor
                                        editor.update_motion_paths(&updated_sequence);

                                        editor.canvas_hidden = false;

                                        save_saved_state_raw(editor.saved_state.clone().expect("Couldn't get saved state"));
                                        
                                        println!("Animation data successfully integrated into sequence (overwrote existing)");
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Process export progress updates
                // uses tokio, not std
                if let Ok(mut rx) = export_progress_rx_for_render.try_lock() {
                    while let Ok(progress) = rx.try_recv() {
                        match progress {
                            ExportProgress::Progress(percent) => {
                                export_progress.set(percent);
                                export_status.set(format!("Exporting: {:.1}%", percent));
                                export_button_text.set(format!("Exporting {:.0}%", percent));
                            }
                            ExportProgress::Complete(output_path) => {
                                export_progress.set(100.0);
                                export_status.set("Export complete!".to_string());
                                export_button_text.set("Export".to_string());
                                is_exporting.set(false);
                                
                                // Open the output folder in explorer
                                tokio::spawn(async move {
                                    if let Err(e) = std::process::Command::new("explorer")
                                        .arg(&output_path)
                                        .spawn() 
                                    {
                                        println!("Failed to open file browser: {}", e);
                                    }
                                });
                            }
                            ExportProgress::Error(err) => {
                                export_status.set(format!("Export failed: {}", err));
                                export_button_text.set("Export".to_string());
                                is_exporting.set(false);
                            }
                        }
                    }
                }
                
                // Process any pending commands from the UI thread
                if let Ok(rx) = command_rx_for_render.try_lock() {
                    while let Ok(command) = rx.try_recv() {
                        if let Ok(mut editor) = editor_for_render.try_lock() {
                            if let Ok(mut editor_state) = state_for_render.try_lock() {
                                let selected_project = selected_project_signal.get().unwrap_or(ProjectData {
                                    project_id: Uuid::new_v4().to_string(),
                                    project_name: "Secret name".to_string()
                                });

                                match command {
                                    Command::AddMotion => {
                                        println!("Processing add motion command from channel");
                                        editor.motion_mode = true;
                                        println!("Motion mode enabled - user can now place arrows by clicking and dragging");
                                    }
                                    Command::AddSquarePolygon => {
                                        println!("Processing add square polygon command from channel");
                                        let random_coords = get_random_coords(window_size);
                                        let new_id = Uuid::new_v4();

                                        let polygon_config = PolygonConfig {
                                            id: new_id,
                                            name: "Square".to_string(),
                                            points: vec![
                                                Point { x: 0.0, y: 0.0 },
                                                Point { x: 1.0, y: 0.0 },
                                                Point { x: 1.0, y: 1.0 },
                                                Point { x: 0.0, y: 1.0 },
                                            ],
                                            dimensions: (100.0, 100.0),
                                            position: Point {
                                                x: random_coords.0 as f32,
                                                y: random_coords.1 as f32,
                                            },
                                            border_radius: 0.0,
                                            fill: [1.0, 0.0, 0.0, 1.0], // Red color
                                            stroke: Stroke {
                                                fill: [0.0, 0.0, 0.0, 1.0], // Black border
                                                thickness: 2.0,
                                            },
                                            layer: 2,
                                        };

                                        // println!("add polyong {:?}", current_sequence_id.get().clone());
                                        
                                        editor.add_polygon(
                                            polygon_config.clone(),
                                            polygon_config.name.clone(),
                                            polygon_config.id,
                                            current_sequence_id.get().clone(),
                                        );

                                        editor_state.add_saved_polygon(
                                            &mut editor.saved_state.as_mut().expect("Couldn't get saved state"),
                                            current_sequence_id.get().clone(),
                                            SavedPolygonConfig {
                                                id: polygon_config.id.to_string().clone(),
                                                name: polygon_config.name.clone(),
                                                dimensions: (
                                                    polygon_config.dimensions.0 as i32,
                                                    polygon_config.dimensions.1 as i32,
                                                ),
                                                fill: [
                                                    polygon_config.fill[0] as i32,
                                                    polygon_config.fill[1] as i32,
                                                    polygon_config.fill[2] as i32,
                                                    polygon_config.fill[3] as i32,
                                                ],
                                                border_radius: polygon_config.border_radius as i32, // multiply by 100?
                                                position: SavedPoint {
                                                    x: polygon_config.position.x as i32,
                                                    y: polygon_config.position.y as i32,
                                                },
                                                stroke: SavedStroke {
                                                    thickness: polygon_config.stroke.thickness as i32,
                                                    fill: [
                                                        polygon_config.stroke.fill[0] as i32,
                                                        polygon_config.stroke.fill[1] as i32,
                                                        polygon_config.stroke.fill[2] as i32,
                                                        polygon_config.stroke.fill[3] as i32,
                                                    ],
                                                },
                                                layer: polygon_config.layer.clone(),
                                            },
                                        );

                                        let saved_state = editor
                                            .saved_state
                                            .as_ref()
                                            .expect("Couldn't get saved state");
                                        let updated_sequence = saved_state
                                            .sequences
                                            .iter()
                                            .find(|s| s.id == current_sequence_id.get().clone())
                                            .expect("Couldn't get updated sequence");
                                        
                                        let sequence_cloned = updated_sequence.clone();
                                        
                                        // drop(editor_state);

                                        // let mut editor = editor_cloned.lock().unwrap();

                                        editor.current_sequence_data = Some(sequence_cloned.clone());
                                        editor.update_motion_paths(&sequence_cloned);
                                        
                                        // drop(editor);

                                        println!("Square polygon added to editor successfully: {}", polygon_config.id);
                                    }
                                    Command::AddText => {
                                        println!("Processing add text command from channel");
                                        let random_coords = get_random_coords(window_size);
                                        let new_id = Uuid::new_v4();

                                        let text_config = TextRendererConfig {
                                            id: new_id,
                                            name: "Text".to_string(),
                                            text: "Sample Text".to_string(),
                                            font_family: "Aleo".to_string(),
                                            dimensions: (200.0, 50.0),
                                            position: Point {
                                                x: random_coords.0 as f32,
                                                y: random_coords.1 as f32,
                                            },
                                            layer: 2,
                                            color: [0, 0, 0, 255],
                                            font_size: 24,
                                            background_fill: [255, 255, 255, 255],
                                        };

                                        let window_size = WindowSize {
                                            width: window_size.width,
                                            height: window_size.height,
                                        };

                                        editor.add_text_item(
                                            &window_size,
                                            device,
                                            queue,
                                            text_config.clone(),
                                            text_config.text.clone(),
                                            new_id,
                                            current_sequence_id.get().clone(),
                                        );

                                        editor_state.add_saved_text_item(
                                            &mut editor.saved_state.as_mut().expect("Couldn't get saved state"),
                                            current_sequence_id.get().clone(),
                                            SavedTextRendererConfig {
                                                id: text_config.id.to_string(),
                                                name: text_config.name.clone(),
                                                text: text_config.text.clone(),
                                                font_family: text_config.font_family.clone(),
                                                dimensions: (text_config.dimensions.0 as i32, text_config.dimensions.1 as i32),
                                                position: SavedPoint {
                                                    x: text_config.position.x as i32,
                                                    y: text_config.position.y as i32,
                                                },
                                                layer: text_config.layer,
                                                color: text_config.color,
                                                font_size: text_config.font_size,
                                                background_fill: Some(text_config.background_fill),
                                            },
                                        );

                                        let saved_state = editor
                                            .saved_state
                                            .as_ref()
                                            .expect("Couldn't get saved state");
                                        let updated_sequence = saved_state
                                            .sequences
                                            .iter()
                                            .find(|s| s.id == current_sequence_id.get().clone())
                                            .expect("Couldn't get updated sequence");
                                        
                                        let sequence_cloned = updated_sequence.clone();
                                        
                                        editor.current_sequence_data = Some(sequence_cloned.clone());
                                        editor.update_motion_paths(&sequence_cloned);

                                        println!("Text item added to editor successfully: {}", text_config.id);
                                    }
                                    Command::AddImage { file_path } => {
                                        println!("Processing add image command from channel with file: {}", file_path);
                                        let random_coords = get_random_coords(window_size);
                                        let new_id = Uuid::new_v4();
                                        
                                        // Extract filename for a better name
                                        let filename = std::path::Path::new(&file_path)
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("Image");

                                        let image_config = StImageConfig {
                                            id: new_id.to_string(),
                                            name: filename.to_string(),
                                            path: file_path.clone(),
                                            dimensions: (150, 150),
                                            position: Point {
                                                x: random_coords.0 as f32,
                                                y: random_coords.1 as f32,
                                            },
                                            layer: 2,
                                        };

                                        let window_size = WindowSize {
                                            width: window_size.width,
                                            height: window_size.height,
                                        };

                                        editor.add_image_item(
                                            &window_size,
                                            device,
                                            queue,
                                            image_config.clone(),
                                            &Path::new(&file_path.clone()),
                                            new_id,
                                            current_sequence_id.get().clone(),
                                        );

                                        editor_state.add_saved_image_item(
                                            &mut editor.saved_state.as_mut().expect("Couldn't get saved state"),
                                            current_sequence_id.get().clone(),
                                            SavedStImageConfig {
                                                id: image_config.id.clone(),
                                                name: image_config.name.clone(),
                                                path: file_path,
                                                dimensions: image_config.dimensions,
                                                position: SavedPoint {
                                                    x: image_config.position.x as i32,
                                                    y: image_config.position.y as i32,
                                                },
                                                layer: image_config.layer,
                                            },
                                        );

                                        let saved_state = editor
                                            .saved_state
                                            .as_ref()
                                            .expect("Couldn't get saved state");
                                        let updated_sequence = saved_state
                                            .sequences
                                            .iter()
                                            .find(|s| s.id == current_sequence_id.get().clone())
                                            .expect("Couldn't get updated sequence");
                                        
                                        let sequence_cloned = updated_sequence.clone();
                                        
                                        editor.current_sequence_data = Some(sequence_cloned.clone());
                                        editor.update_motion_paths(&sequence_cloned);

                                        println!("Image item added to editor successfully: {}", image_config.id);
                                    }
                                    Command::AddVideo { file_path } => {
                                        println!("Processing add video command from channel with file: {}", file_path);
                                        let random_coords = get_random_coords(window_size);
                                        let new_id = Uuid::new_v4();
                                        
                                        let path = std::path::Path::new(&file_path);
                                        // Extract filename for a better name
                                        let filename = path
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("Video");

                                        let parent = path.parent().unwrap_or(Path::new(""));
    
                                        let mouse_positions_path = parent.join("mousePositions.json");
                                        let source_data_path = parent.join("sourceData.json");

                                        let video_config = StVideoConfig {
                                            id: new_id.to_string(),
                                            name: filename.to_string(),
                                            path: file_path.clone(),
                                            dimensions: (300, 200),
                                            position: Point {
                                                x: random_coords.0 as f32,
                                                y: random_coords.1 as f32,
                                            },
                                            layer: 2,
                                            mouse_path: Some(mouse_positions_path.clone().display().to_string()),
                                        };

                                        let window_size = WindowSize {
                                            width: window_size.width,
                                            height: window_size.height,
                                        };

                                        let mut saved_mouse_path = None;
                                        let mut stored_mouse_positions = None;
                                        // if let Some(mouse_path) = &mouse_positions_path {
                                            if let Ok(positions) = fs::read_to_string(mouse_positions_path.clone()) {
                                                if let Ok(mouse_positions) = serde_json::from_str::<Vec<MousePosition>>(&positions) {
                                                    let the_path = mouse_positions_path.to_str().expect("Couldn't make string from path");
                                                    saved_mouse_path = Some(the_path.to_string());
                                                    stored_mouse_positions = Some(mouse_positions);
                                                }
                                            }
                                        // }

                                        let mut stored_source_data = None;
                                        // if let Some(source_path) = &source_data_path {
                                            if let Ok(source_data) = fs::read_to_string(source_data_path) {
                                                if let Ok(data) = serde_json::from_str::<SourceData>(&source_data) {
                                                    stored_source_data = Some(data);
                                                }
                                            }
                                        // }

                                        editor.add_video_item(
                                            &window_size,
                                            device,
                                            queue,
                                            video_config.clone(),
                                            &Path::new(&file_path.clone()),
                                            new_id,
                                            current_sequence_id.get().clone(),
                                            stored_mouse_positions, // stored_mouse_positions
                                            stored_source_data, // stored_source_data
                                        );

                                        let source_duration_ms = editor
                                            .video_items
                                            .last()
                                            .expect("Couldn't get latest video")
                                            .source_duration_ms
                                            .clone();

                                        editor_state.add_saved_video_item(
                                            &mut editor.saved_state.as_mut().expect("Couldn't get saved state"),
                                            current_sequence_id.get().clone(),
                                            SavedStVideoConfig {
                                                id: video_config.id.clone(),
                                                name: video_config.name.clone(),
                                                path: file_path,
                                                dimensions: video_config.dimensions,
                                                position: SavedPoint {
                                                    x: video_config.position.x as i32,
                                                    y: video_config.position.y as i32,
                                                },
                                                layer: video_config.layer,
                                                mouse_path: Some(mouse_positions_path.clone().display().to_string()),
                                            },
                                            source_duration_ms
                                        );

                                        let saved_state = editor
                                            .saved_state
                                            .as_ref()
                                            .expect("Couldn't get saved state");
                                        let updated_sequence = saved_state
                                            .sequences
                                            .iter()
                                            .find(|s| s.id == current_sequence_id.get().clone())
                                            .expect("Couldn't get updated sequence");
                                        
                                        let sequence_cloned = updated_sequence.clone();
                                        
                                        editor.current_sequence_data = Some(sequence_cloned.clone());
                                        editor.update_motion_paths(&sequence_cloned);

                                        println!("Video item added to editor successfully: {}", video_config.id);
                                    }
                                    Command::SubmitMotionForm { description, position, scale, opacity, rotation, delay, duration } => {
                                        println!("Processing motion form submission from channel");

                                        // Reset canvas hidden state
                                        // let mut editor_lock = editor_for_render.lock().unwrap();
                                        // editor.canvas_hidden = false;
                                        let object_dimensions = editor.last_motion_arrow_object_dimensions.clone();
                                        let arrow_positions = editor.last_motion_arrow_end_positions.clone();
                                        // drop(editor_lock);
                                        
                                        // println!("Motion form submitted - Description: {}, Position: {}, Scale: {}, Opacity: {}, Dimensions: {:?}", 
                                        // description.get(), position.get(), scale.get(), opacity.get(), object_dimensions);
                                
                                        display_motion_loading.set(true);
                                        
                                        // Prepare API data
                                        let api_data = serde_json::json!({
                                            "description": description,
                                            "position": position,
                                            "scale": scale,
                                            "opacity": opacity,
                                            "rotation": rotation,
                                            "delay": delay,
                                            "duration": duration,
                                            "arrow_positions": arrow_positions.map(|(p1, p2)| serde_json::json!({"startX": p1.x, "startY": p1.y, "endX": p2.x, "endY": p2.y})),
                                            "object_dimensions": object_dimensions.map(|(w, h)| serde_json::json!({"width": w, "height": h}))
                                        });
                                        
                                        // Clone the response sender for the async block
                                        let response_sender = api_response_tx_for_render.clone();
                                        
                                        // Get the polygon_id before the async block to avoid Send issues
                                        let polygon_id = editor.last_motion_arrow_object_id.to_string();
                                        let object_type = editor.last_motion_arrow_object_type.clone();

                                        let display_motion_form = display_motion_form.clone();
                                        let display_motion_loading = display_motion_loading.clone();

                                        println!("api_data {:?} {:?} {:?}", api_data, polygon_id, object_dimensions);
                                        
                                        // Spawn the async task - no editor locking here!
                                        tokio::spawn(async move {
                                            let client = reqwest::Client::new();
                                            match client
                                                .post("http://localhost:3000/api/projects/generate-motion")
                                                .json(&api_data)
                                                .send()
                                                .await
                                            {
                                                Ok(response) => {
                                                    if response.status().is_success() {
                                                        // Get the raw response text first for logging
                                                        match response.text().await {
                                                            Ok(raw_response) => {
                                                                println!("Raw API response received:");
                                                                println!("{}", raw_response);

                                                                display_motion_form.set(false);
                                                                display_motion_loading.set(false);
                                                                
                                                                // First try to parse as API format, then convert
                                                                match serde_json::from_str::<ApiAnimationData>(&raw_response) {
                                                                    Ok(api_data) => {
                                                                        println!("Successfully parsed API response format: {:?}", api_data);
                                                                        
                                                                        // Convert to the expected format
                                                                        let animation_data = api_data.to_animation_data(polygon_id, object_type);
                                                                        println!("Converted to AnimationData format: {:?}", animation_data);
                                                                        
                                                                        // Send the response back through the channel
                                                                        if let Err(e) = response_sender.send(animation_data) {
                                                                            println!("Failed to send API response through channel: {}", e);
                                                                        }
                                                                    }
                                                                    Err(parse_error) => {
                                                                        println!("Failed to parse API response as ApiAnimationData:");
                                                                        println!("Parse error: {}", parse_error);
                                                                        println!("Raw response that failed to parse: {}", raw_response);
                                                                        
                                                                        // Try direct parsing as fallback and show expected formats
                                                                        println!("\nTrying direct AnimationData parsing for comparison...");
                                                                        match serde_json::from_str::<stunts_engine::animations::AnimationData>(&raw_response) {
                                                                            Ok(_) => println!("Direct AnimationData parsing succeeded (unexpected!)"),
                                                                            Err(direct_error) => {
                                                                                println!("Direct AnimationData parsing also failed: {}", direct_error);
                                                                                
                                                                                // Log expected structures for comparison
                                                                                println!("\n=== Expected API Format Example ===");
                                                                                let example_api = ApiAnimationData {
                                                                                    id: "example_id".to_string(),
                                                                                    duration: 3000,
                                                                                    properties: vec![
                                                                                        ApiAnimationProperty {
                                                                                            name: "Position".to_string(),
                                                                                            keyframes: vec![
                                                                                                ApiKeyframe {
                                                                                                    time: 0,
                                                                                                    value: ApiKeyframeValue::Position { Position: ApiPosition { x: 100, y: 100 } },
                                                                                                    easing: "Linear".to_string(),
                                                                                                }
                                                                                            ],
                                                                                        }
                                                                                    ],
                                                                                };
                                                                                if let Ok(example_json) = serde_json::to_string_pretty(&example_api) {
                                                                                    println!("{}", example_json);
                                                                                }
                                                                                
                                                                                println!("\n=== Expected AnimationData Format Example ===");
                                                                                let example_animation_data = stunts_engine::animations::AnimationData::default();
                                                                                if let Ok(example_json) = serde_json::to_string_pretty(&example_animation_data) {
                                                                                    println!("{}", example_json);
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(text_error) => {
                                                                println!("Failed to get response text: {}", text_error);
                                                            }
                                                        }
                                                    } else {
                                                        println!("API call failed with status: {}", response.status());
                                                        display_motion_form.set(false);
                                                        display_motion_loading.set(false);
                                                    }
                                                }
                                                Err(e) => println!("API call failed: {}", e),
                                            }
                                        });
                                    }
                                    Command::UpdateTextProperty { property_key, property_value } => {
                                        println!("Processing text property update: {} = {}", property_key, property_value);
                                        
                                        // Convert property to ObjectProperty enum
                                        let object_property = match property_key.as_str() {
                                            "font_family" => ObjectProperty::FontFamily(property_value),
                                            "font_size" => {
                                                if let Ok(size) = property_value.parse::<f32>() {
                                                    ObjectProperty::FontSize(size)
                                                } else {
                                                    println!("Invalid font size: {}", property_value);
                                                    continue;
                                                }
                                            },
                                            "text_content" => ObjectProperty::Text(property_value),
                                            _ => {
                                                println!("Unsupported property: {}", property_key);
                                                continue;
                                            }
                                        };

                                        // if let Some(last_text) = editor.text_items.last() {
                                            let text_id = editor.selected_polygon_id; //  this will be currently selected text object id
                                            let window_size = WindowSize {
                                                width: window_size.width,
                                                height: window_size.height,
                                            };
                                            
                                            if let Err(e) = editor.update_text_property(
                                                text_id,
                                                object_property,
                                            ) {
                                                println!("Failed to update text property: {}", e);
                                            } else {
                                                println!("Text property updated successfully");
                                            }
                                        // } else {
                                        //     println!("No text items to update");
                                        // }
                                    }
                                    Command::TogglePlay => {
                                        if editor.is_playing {
                                            println!("Pause Sequence...");

                                            editor.is_playing = false;
                                            editor.start_playing_time = None;

                                            // should return objects to the startup positions and state
                                            editor.reset_sequence_objects();
                                        } else {
                                            println!("Play Sequence...");

                                            let now = std::time::Instant::now();
                                            editor.start_playing_time = Some(now);
                                            editor.is_playing = true;
                                        }
                                    }
                                    Command::ShowCaptureSources => {
                                        println!("Processing show capture sources command");
                                        // Get available capture sources (Windows enumeration)
                                        // Note: This runs in render thread where editor is accessible
                                        match get_sources() {
                                            Ok(sources) => {
                                                let filtered_sources: Vec<WindowInfo> = sources
                                                    .into_iter()
                                                    .filter(|s| s.title.len() > 1 && s.rect.width > 100 && s.rect.height > 100)
                                                    .collect();

                                                // Store sources for UI (can't directly update UI from here)
                                                // The UI will need to be rebuilt with source buttons
                                                println!("Found {} valid capture sources", filtered_sources.len());
                                                for source in &filtered_sources {
                                                    println!("Source: {} (HWND: {}) - {}x{}",
                                                        source.title, source.hwnd, source.rect.width, source.rect.height);
                                                }

                                                // Show the dropdown
                                                capture_sources_visible.set(true);

                                                    

                                                // Update the signal with available sources
                                                available_capture_sources.set(filtered_sources.into_iter().map(|source| {
                                                    DropdownOption {
                                                        label: format!("{} ({}x{})", source.title, source.rect.width, source.rect.height),
                                                        // value: source.hwnd.to_string(),
                                                        value: format!("{}-{}x{}", source.hwnd.to_string(), source.rect.width, source.rect.height)
                                                    }
                                                }).collect());
                                            }
                                            Err(e) => {
                                                println!("Failed to get capture sources: {}", e);
                                            }
                                        }
                                    }
                                    Command::StartScreenCapture { hwnd, width, height } => {
                                        println!("Processing start screen capture command for HWND: {}", hwnd);

                                        editor.st_capture
                                            .start_mouse_tracking()
                                            .expect(
                                                "Couldn't start mouse tracking",
                                            );

                                        match editor.st_capture.start_video_capture(hwnd, width as u32, height as u32, project_id.to_string()) {
                                            Ok(_) => {
                                                println!("Screen capture started successfully");
                                                is_recording.set(true);
                                                capture_button_text.set("Stop Capture".to_string());
                                                capture_sources_visible.set(false);
                                            }
                                            Err(e) => {
                                                println!("Failed to start screen capture: {}", e);
                                            }
                                        }
                                    }
                                    Command::StopScreenCapture => {
                                        println!("Processing stop screen capture command");

                                        let (mouse_positions_path) = editor.st_capture
                                                    .stop_mouse_tracking(
                                                        project_id.to_string(),
                                                    )
                                                    .expect(
                                                        "Couldn't stop mouse tracking",
                                                    );

                                        match editor.st_capture.stop_video_capture(project_id.to_string()) {
                                            Ok((video_path, mouse_data_path)) => {
                                                println!("Screen capture stopped successfully");
                                                println!("Video saved to: {:?}", video_path);
                                                println!("Mouse data saved to: {:?}", mouse_data_path);

                                                is_recording.set(false);
                                                capture_button_text.set("Screen Capture".to_string());

                                                // Video will be automatically added via the completion callback
                                            }
                                            Err(e) => {
                                                println!("Failed to stop screen capture: {}", e);
                                                is_recording.set(false);
                                                capture_button_text.set("Screen Capture".to_string());
                                            }
                                        }
                                    }
                                    Command::Export { progress_tx } => {
                                        println!("Processing export command");
                                        
                                        if is_exporting.get() {
                                            println!("Export already in progress");
                                            continue;
                                        }
                                        
                                        is_exporting.set(true);
                                        export_status.set("Starting export...".to_string());
                                        export_button_text.set("Exporting...".to_string());
                                        
                                        // Get all sequences from editor.saved_state
                                        let sequences = if let Some(ref saved_state) = editor.saved_state {
                                            saved_state.sequences.clone()
                                        } else {
                                            println!("No saved state found");
                                            export_status.set("No sequences to export".to_string());
                                            export_button_text.set("Export".to_string());
                                            is_exporting.set(false);
                                            continue;
                                        };
                                        
                                        if sequences.is_empty() {
                                            println!("No sequences to export");
                                            export_status.set("No sequences to export".to_string());
                                            export_button_text.set("Export".to_string());
                                            is_exporting.set(false);
                                            continue;
                                        }
                                        
                                        // Create default timeline config with sequences in series
                                        let mut timeline_sequences = Vec::new();
                                        let mut current_start_time = 0;
                                        
                                        for sequence in &sequences {
                                            timeline_sequences.push(TimelineSequence {
                                                id: Uuid::new_v4().to_string(),
                                                sequence_id: sequence.id.clone(),
                                                start_time_ms: current_start_time,
                                                // duration_ms: sequence.duration_ms,
                                                track_type: stunts_engine::timelines::TrackType::Video,
                                            });
                                            current_start_time += sequence.duration_ms;
                                        }
                                        
                                        let timeline_config = SavedTimelineStateConfig {
                                            timeline_sequences,
                                        };
                                        
                                        // Calculate total duration
                                        let total_duration_s = sequences.iter()
                                            .map(|s| s.duration_ms as f64 / 1000.0)
                                            .sum::<f64>();
                                        
                                        if total_duration_s <= 0.0 {
                                            println!("Invalid sequence duration");
                                            export_status.set("Invalid sequence duration".to_string());
                                            export_button_text.set("Export".to_string());
                                            is_exporting.set(false);
                                            continue;
                                        }
                                        
                                        // Create exports directory and filename
                                        let exports_dir = std::env::current_dir()
                                            .unwrap_or_else(|_| std::path::PathBuf::from("."))
                                            .join("exports");

                                        if let Err(e) = std::fs::create_dir_all(&exports_dir) {
                                            println!("Failed to create exports directory: {}", e);
                                            export_status.set("Failed to create exports directory".to_string());
                                            export_button_text.set("Export".to_string());
                                            is_exporting.set(false);
                                            continue;
                                        }

                                        let filename = format!("export_{}.mp4", 
                                            std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap()
                                                .as_secs());
                                        
                                        let output_path = exports_dir.join(filename);
                                        let output_path_str = output_path.to_str().expect("Invalid export path").to_string();
                                        
                                        // Get window size for export
                                        let export_window_size = WindowSize {
                                            // width: window_size.width,
                                            // height: window_size.height,
                                            width: 1920,
                                            height: 1080
                                        };
                                        
                                        let project_id_for_export = project_id.to_string();
                                        
                                        println!("Starting export thread - sequences: {}, duration: {}s", 
                                                sequences.len(), total_duration_s);
                                        
                                        // Spawn export thread
                                        std::thread::spawn(move || {
                                            // Create tokio runtime for this thread
                                            let rt = tokio::runtime::Runtime::new().unwrap();
                                            
                                            rt.block_on(async {
                                                // Create exporter in the export thread
                                                let mut exporter = Exporter::new(&output_path_str);
                                                
                                                match exporter.run(
                                                    export_window_size,
                                                    sequences,
                                                    timeline_config,
                                                    export_window_size.width,
                                                    export_window_size.height,
                                                    total_duration_s,
                                                    progress_tx.clone(),
                                                    project_id_for_export,
                                                ).await {
                                                    Ok(_) => {
                                                        println!("Export completed successfully");
                                                        let _ = progress_tx.send(ExportProgress::Complete(output_path_str));
                                                    }
                                                    Err(e) => {
                                                        println!("Export failed: {}", e);
                                                        let _ = progress_tx.send(ExportProgress::Error(e));
                                                    }
                                                }
                                            });
                                        });
                                    }
                                    // Authentication commands
                                    Command::SubmitSignIn { email, password } => {
                                        println!("Processing sign in command");
                                        auth_loading.set(true);
                                        
                                        let auth_state = auth_state.clone();
                                        let local_projects_signal = local_projects_signal.clone();
                                        let show_auth_form = show_auth_form.clone();
                                        let show_project_list = show_project_list.clone();
                                        let auth_loading = auth_loading.clone();
                                        let editor_for_auth = editor_for_render.clone();
                                        
                                        tokio::spawn(async move {
                                            match authenticate_user(&email, &password).await {
                                                Ok(auth_response) => {
                                                    // Create AuthToken with expiry
                                                    let auth_token = AuthToken {
                                                        token: auth_response.jwt_data.token.clone(),
                                                        expiry: Some(chrono::Utc::now() + chrono::Duration::seconds(auth_response.jwt_data.expiry)),
                                                    };
                                                    
                                                    // Store token securely in keyring
                                                    if let Err(e) = store_auth_token(&auth_token) {
                                                        println!("Failed to store auth token: {}", e);
                                                    }
                                                    
                                                    // Fetch subscription details
                                                    match helpers::utilities::fetch_subscription_details(&auth_response.jwt_data.token).await {
                                                        Ok(subscription) => {
                                                            let new_auth_state = AuthState {
                                                                token: Some(auth_token),
                                                                is_authenticated: true,
                                                                subscription: Some(subscription),
                                                            };
                                                            
                                                            // Load local projects
                                                            match load_local_projects() {
                                                                Ok(projects) => {
                                                                    auth_state.set(new_auth_state);
                                                                    local_projects_signal.set(projects);
                                                                    auth_loading.set(false);
                                                                    show_auth_form.set(false);
                                                                    show_project_list.set(true);
                                                                    
                                                                    println!("Authentication successful, loaded local projects");
                                                                }
                                                                Err(e) => {
                                                                    println!("Failed to load local projects: {}", e);
                                                                    auth_loading.set(false);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            println!("Failed to fetch subscription details: {}", e);
                                                            auth_loading.set(false);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("Authentication failed: {}", e);
                                                    auth_loading.set(false);
                                                }
                                            }
                                        });
                                    }
                                    Command::SignOut => {
                                        println!("Processing sign out command");
                                        
                                        // Clear stored token
                                        let _ = clear_stored_auth_token();
                                        
                                        // Reset authentication state
                                        let new_auth_state = AuthState {
                                            token: None,
                                            is_authenticated: false,
                                            subscription: None,
                                        };
                                        auth_state.set(new_auth_state);
                                        local_projects_signal.set(Vec::new());
                                        selected_project_signal.set(None);
                                        
                                        // Update UI visibility
                                        show_project_list.set(false);
                                        show_project_creation.set(false);
                                        show_auth_form.set(true);
                                        show_editor.set(false);
                                        
                                        // Hide canvas
                                        editor.canvas_hidden = true;
                                        
                                        println!("User signed out successfully");
                                    }
                                    Command::LoadProjects => {
                                        println!("Processing load projects command");
                                        
                                        match load_local_projects() {
                                            Ok(projects) => {
                                                local_projects_signal.set(projects);
                                                println!("Local projects reloaded successfully");
                                            }
                                            Err(e) => {
                                                println!("Failed to reload local projects: {}", e);
                                            }
                                        }
                                    }
                                    Command::SelectProject { project_id } => {
                                        println!("Processing select project command: {}", project_id);
                                        
                                        let current_projects = local_projects_signal.get();
                                        if let Some(project) = current_projects.iter().find(|p| p.project_id == project_id) {
                                            selected_project_signal.set(Some(project.clone()));
                                            
                                            // Load the project state
                                            match stunts_engine::saved_state::load_project_state(project_id.clone()) {
                                                Ok(saved_state) => {
                                                    editor.saved_state = Some(saved_state.clone());
                                                    editor.project_selected = Some(uuid::Uuid::parse_str(&project_id).unwrap());
                                                    editor.current_view = "scene".to_string();

                                                    saved_state.sequences.iter().enumerate().for_each(|(i, s)| {
                                                        editor.restore_sequence_objects(
                                                            &s,
                                                            true,
                                                        );
                                                    });
                                                    
                                                    // Set the current sequence data
                                                    if let Some(sequence) = saved_state.sequences.first() {
                                                        current_sequence_id.set(sequence.id.clone());
                                                        editor.current_sequence_data = Some(sequence.clone());

                                                        editor.polygons.iter_mut().for_each(|p| {
                                                            p.hidden = true;
                                                        });
                                                        editor.image_items.iter_mut().for_each(|i| {
                                                            i.hidden = true;
                                                        });
                                                        editor.text_items.iter_mut().for_each(|t| {
                                                            t.hidden = true;
                                                        });
                                                        editor.video_items.iter_mut().for_each(|t| {
                                                            t.hidden = true;
                                                        });

                                                        sequence.active_polygons.iter().for_each(|ap| {
                                                            let polygon = editor
                                                                .polygons
                                                                .iter_mut()
                                                                .find(|p| p.id.to_string() == ap.id)
                                                                .expect("Couldn't find polygon");
                                                            polygon.hidden = false;
                                                        });
                                                        sequence.active_image_items.iter().for_each(|si| {
                                                            let image = editor
                                                                .image_items
                                                                .iter_mut()
                                                                .find(|i| i.id.to_string() == si.id)
                                                                .expect("Couldn't find image");
                                                            image.hidden = false;
                                                        });
                                                        sequence.active_text_items.iter().for_each(|tr| {
                                                            let text = editor
                                                                .text_items
                                                                .iter_mut()
                                                                .find(|t| t.id.to_string() == tr.id)
                                                                .expect("Couldn't find image");
                                                            text.hidden = false;
                                                        });
                                                        sequence.active_video_items.iter().for_each(|tr| {
                                                            let video = editor
                                                                .video_items
                                                                .iter_mut()
                                                                .find(|t| t.id.to_string() == tr.id)
                                                                .expect("Couldn't find image");
                                                            video.hidden = false;
                                                        });

                                                        let mut background_fill = Some(BackgroundFill::Color([
                                                            wgpu_to_human(0.8) as i32,
                                                            wgpu_to_human(0.8) as i32,
                                                            wgpu_to_human(0.8) as i32,
                                                            255,
                                                        ]));

                                                        if sequence.background_fill.is_some() {
                                                            background_fill = sequence.background_fill.clone();
                                                        }

                                                        match background_fill.expect("Couldn't get default background fill")
                                                        {
                                                            BackgroundFill::Color(fill) => {
                                                                editor.replace_background(
                                                                    uuid::Uuid::parse_str(&sequence.id).unwrap(),
                                                                    rgb_to_wgpu(
                                                                        fill[0] as u8,
                                                                        fill[1] as u8,
                                                                        fill[2] as u8,
                                                                        fill[3] as f32,
                                                                    ),
                                                                );
                                                            }
                                                            _ => {
                                                                println!("Not supported yet...");
                                                            }
                                                        }

                                                        editor.update_motion_paths(sequence);
                                                    }
                                                    
                                                    // Hide project selection UI and show main canvas
                                                    show_project_list.set(false);
                                                    show_project_creation.set(false);
                                                    show_editor.set(true);
                                                    editor.canvas_hidden = false;
                                                    
                                                    println!("Project selected and loaded: {}", project.project_name);
                                                }
                                                Err(e) => {
                                                    println!("Failed to load project state: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    Command::CreateProject { name } => {
                                        println!("Processing create project command: {}", name);
                                        
                                        // Check if user can create projects based on subscription
                                        let auth_state = auth_state.get();
                                        if auth_state.can_create_projects() {
                                            match create_local_project(&name) {
                                                Ok(new_project) => {
                                                    // Update local projects list
                                                    let mut current_projects = local_projects_signal.get();
                                                    current_projects.push(new_project.clone());
                                                    local_projects_signal.set(current_projects);
                                                    
                                                    // Set as selected project
                                                    selected_project_signal.set(Some(new_project.clone()));
                                                    
                                                    // Load the newly created project state
                                                    match stunts_engine::saved_state::load_project_state(new_project.project_id.clone()) {
                                                        Ok(mut saved_state) => {
                                                            // Create first sequence automatically
                                                            let sequence_id = Uuid::new_v4();
                                                            let first_sequence = Sequence {
                                                                id: sequence_id.to_string(),
                                                                name: "Sequence 1".to_string(),
                                                                background_fill: Some(BackgroundFill::Color([
                                                                    wgpu_to_human(0.8) as i32,
                                                                    wgpu_to_human(0.8) as i32,
                                                                    wgpu_to_human(0.8) as i32,
                                                                    255,
                                                                ])),
                                                                duration_ms: 20000,
                                                                active_polygons: Vec::new(),
                                                                polygon_motion_paths: Vec::new(),
                                                                active_text_items: Vec::new(),
                                                                active_image_items: Vec::new(),
                                                                active_video_items: Vec::new(),
                                                            };

                                                            current_sequence_id.set(sequence_id.clone().to_string());
                                                            
                                                            saved_state.sequences = vec![first_sequence];
                                                            saved_state.timeline_state = arrange_sequences_in_series(&mut saved_state.sequences);
                                                            
                                                            // Save the updated state
                                                            let _ = stunts_engine::saved_state::save_saved_state_raw(saved_state.clone());
                                                            
                                                            // Update editor with new project and sequence
                                                            editor.saved_state = Some(saved_state.clone());
                                                            editor.project_selected = Some(uuid::Uuid::parse_str(&new_project.project_id).unwrap());
                                                            editor.current_view = "scene".to_string();

                                                            saved_state.sequences.iter().enumerate().for_each(|(i, s)| {
                                                                editor.restore_sequence_objects(
                                                                    &s,
                                                                    true,
                                                                );
                                                            });
                                                            
                                                            // Set the current sequence data
                                                            if let Some(sequence) = saved_state.sequences.first() {
                                                                editor.current_sequence_data = Some(sequence.clone());

                                                                editor.polygons.iter_mut().for_each(|p| {
                                                                    p.hidden = true;
                                                                });
                                                                editor.image_items.iter_mut().for_each(|i| {
                                                                    i.hidden = true;
                                                                });
                                                                editor.text_items.iter_mut().for_each(|t| {
                                                                    t.hidden = true;
                                                                });
                                                                editor.video_items.iter_mut().for_each(|t| {
                                                                    t.hidden = true;
                                                                });

                                                                sequence.active_polygons.iter().for_each(|ap| {
                                                                    let polygon = editor
                                                                        .polygons
                                                                        .iter_mut()
                                                                        .find(|p| p.id.to_string() == ap.id)
                                                                        .expect("Couldn't find polygon");
                                                                    polygon.hidden = false;
                                                                });
                                                                sequence.active_image_items.iter().for_each(|si| {
                                                                    let image = editor
                                                                        .image_items
                                                                        .iter_mut()
                                                                        .find(|i| i.id.to_string() == si.id)
                                                                        .expect("Couldn't find image");
                                                                    image.hidden = false;
                                                                });
                                                                sequence.active_text_items.iter().for_each(|tr| {
                                                                    let text = editor
                                                                        .text_items
                                                                        .iter_mut()
                                                                        .find(|t| t.id.to_string() == tr.id)
                                                                        .expect("Couldn't find image");
                                                                    text.hidden = false;
                                                                });
                                                                sequence.active_video_items.iter().for_each(|tr| {
                                                                    let video = editor
                                                                        .video_items
                                                                        .iter_mut()
                                                                        .find(|t| t.id.to_string() == tr.id)
                                                                        .expect("Couldn't find image");
                                                                    video.hidden = false;
                                                                });

                                                                let mut background_fill = Some(BackgroundFill::Color([
                                                                    wgpu_to_human(0.8) as i32,
                                                                    wgpu_to_human(0.8) as i32,
                                                                    wgpu_to_human(0.8) as i32,
                                                                    255,
                                                                ]));

                                                                if sequence.background_fill.is_some() {
                                                                    background_fill = sequence.background_fill.clone();
                                                                }

                                                                match background_fill.expect("Couldn't get default background fill")
                                                                {
                                                                    BackgroundFill::Color(fill) => {
                                                                        editor.replace_background(
                                                                            uuid::Uuid::parse_str(&sequence.id).unwrap(),
                                                                            rgb_to_wgpu(
                                                                                fill[0] as u8,
                                                                                fill[1] as u8,
                                                                                fill[2] as u8,
                                                                                fill[3] as f32,
                                                                            ),
                                                                        );
                                                                    }
                                                                    _ => {
                                                                        println!("Not supported yet...");
                                                                    }
                                                                }
                                                                
                                                                editor.update_motion_paths(sequence);
                                                            }
                                                            
                                                            // Hide creation form and show main canvas
                                                            show_project_creation.set(false);
                                                            show_project_list.set(false);
                                                            show_editor.set(true);
                                                            project_name_text.set("".to_string());
                                                            editor.canvas_hidden = false;
                                                            
                                                            println!("Project created successfully: {}", new_project.project_name);
                                                        }
                                                        Err(e) => {
                                                            println!("Failed to load newly created project state: {}", e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("Failed to create local project: {}", e);
                                                }
                                            }
                                        }
                                    }
                                    Command::CreateSequence { name, project_id } => {
                                        println!("Processing create sequence command: {} for project {}", name, project_id);
                                        // TODO: Implement sequence creation API call
                                    }
                                    Command::ApplyTheme { theme } => {
                                        println!("Applying theme: {:?}", theme);

                                        let background_color_row = theme[0].trunc() as usize;
                                        let background_color_column = (theme[0].fract() * 10.0) as usize;
                                        let background_color_hex = theme_sidebar::THEME_COLORS[background_color_row][background_color_column];

                                        let text_color_row = theme[4].trunc() as usize;
                                        let text_color_column = (theme[4].fract() * 10.0) as usize;
                                        let text_color_hex = theme_sidebar::THEME_COLORS[text_color_row][text_color_column];

                                        // Parse hex to RGB
                                        let background_color = hex_to_rgb(background_color_hex);
                                        let text_color = hex_to_rgb(text_color_hex);
                                        let font_index = theme[2];

                                        println!("Updating text color: {:?}", text_color);
                                        println!("Updating background color: {:?}", background_color);

                                        let text_color_wgpu = rgb_to_wgpu(
                                            text_color[0] as u8,
                                            text_color[1] as u8,
                                            text_color[2] as u8,
                                            255.0,
                                        );

                                        let background_color_wgpu = rgb_to_wgpu(
                                            background_color[0] as u8,
                                            background_color[1] as u8,
                                            background_color[2] as u8,
                                            255.0,
                                        );

                                        // Update text items for current sequence
                                        let ids_to_update: Vec<_> = editor
                                            .text_items
                                            .iter()
                                            .filter(|text| {
                                                text.current_sequence_id.to_string() == current_sequence_id.get()
                                            })
                                            .map(|text| text.id)
                                            .collect();

                                        if let Some(font_data) = editor.font_manager.font_data.get(font_index as usize) {
                                            let font_id = font_data.0.clone();

                                            for id in ids_to_update.clone() {
                                                editor.update_text_color(id, text_color);
                                                editor.update_text_font_family(font_id.clone(), id);
                                                
                                                // Update text fill colors
                                                editor.update_text(id, "red_fill", stunts_engine::editor::InputValue::Number(text_color_wgpu[0] as f32), false);
                                                editor.update_text(id, "green_fill", stunts_engine::editor::InputValue::Number(text_color_wgpu[1] as f32), false);
                                                editor.update_text(id, "blue_fill", stunts_engine::editor::InputValue::Number(text_color_wgpu[2] as f32), false);
                                            }
                                        }

                                        // loop through polygons and apply text_color (so it contrasts with background)
                                        let ids_to_update: Vec<_> = editor
                                            .polygons
                                            .iter()
                                            .filter(|poly| {
                                                poly.current_sequence_id.to_string() == current_sequence_id.get()
                                            })
                                            .map(|poly| poly.id)
                                            .collect();

                                        for id in ids_to_update.clone() {
                                            editor.update_polygon(id, "red", stunts_engine::editor::InputValue::Number(text_color_wgpu[0] as f32), false);
                                            editor.update_polygon(id, "green", stunts_engine::editor::InputValue::Number(text_color_wgpu[1] as f32), false);
                                            editor.update_polygon(id, "blue", stunts_engine::editor::InputValue::Number(text_color_wgpu[2] as f32), false);
                                        }

                                        // Update background for current sequence
                                        if let Ok(background_uuid) = uuid::Uuid::parse_str(&current_sequence_id.get()) {
                                            editor.update_background(
                                                background_uuid,
                                                "red",
                                                stunts_engine::editor::InputValue::Number(background_color[0] as f32),
                                            );
                                            editor.update_background(
                                                background_uuid,
                                                "green",
                                                stunts_engine::editor::InputValue::Number(background_color[1] as f32),
                                            );
                                            editor.update_background(
                                                background_uuid,
                                                "blue",
                                                stunts_engine::editor::InputValue::Number(background_color[2] as f32),
                                            );
                                        }

                                        // Update saved state
                                        if let Some(saved_state) = editor.saved_state.as_mut() {
                                            saved_state.sequences.iter_mut().for_each(|s| {
                                                if s.id == current_sequence_id.get() {
                                                    // Update text items
                                                    s.active_text_items.iter_mut().for_each(|t| {
                                                        t.color = text_color;
                                                        if let Some(background_fill) = t.background_fill.as_mut() {
                                                            *background_fill = text_color;
                                                        }
                                                    });

                                                    // // Update sequence background
                                                    // if s.background_fill.is_none() {
                                                    //     s.background_fill = Some(BackgroundFill::Color([
                                                    //         wgpu_to_human(0.8) as i32,
                                                    //         wgpu_to_human(0.8) as i32, 
                                                    //         wgpu_to_human(0.8) as i32,
                                                    //         255,
                                                    //     ]));
                                                    // }

                                                    // if let Some(BackgroundFill::Color(fill)) = s.background_fill.as_mut() {
                                                    //     *fill = background_color;
                                                    // }

                                                    // Just set it directly - simpler and clearer
                                                    s.background_fill = Some(BackgroundFill::Color(background_color));

                                                }
                                            });
                                        }

                                        save_saved_state_raw(editor.saved_state.clone().expect("Couldn't get saved state"));

                                        println!("Theme applied successfully!");
                                    }
                                    
                                }
                            }
                        } else {
                            println!("Could not acquire editor lock during render, command not processed");
                        }
                    }
                }
                
                // Create engine handle lazily on first render (after pipeline is initialized)
                let mut cache = engine_handle_cache.borrow_mut();
                if cache.is_none() {
                    if let Some(handle) = render_integration::EngineHandle::try_new(editor_for_render.clone()) {
                        *cache = Some(handle);
                    } else {
                        // Pipeline not ready yet, skip rendering
                        return Ok(());
                    }
                }
                
                render_integration::render_stunts_content(
                    cache.as_ref().unwrap(), 
                    device, 
                    queue, 
                    encoder, 
                    external_resources, 
                    view,
                    sidebar_visible.get(),
                    sidebar_width
                )
            })
        }
    )
}

fn hex_to_rgb(hex: &str) -> [i32; 4] {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = i32::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = i32::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = i32::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        [r, g, b, 255]
    } else {
        [128, 128, 128, 255] // fallback gray
    }
}