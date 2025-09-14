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
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::path::Path;
use stunts_engine::{
    editor::{Viewport, WindowSize, Editor, Point, WindowSizeShader, ObjectProperty},
    capture::{StCapture, get_sources, WindowInfo},
};
use stunts_engine::polygon::{
    Polygon, PolygonConfig, SavedPoint, SavedPolygonConfig, SavedStroke, Stroke,
};
use stunts_engine::st_image::{SavedStImageConfig, StImage, StImageConfig};
use stunts_engine::st_video::{SavedStVideoConfig, StVideoConfig};
use stunts_engine::text_due::{SavedTextRendererConfig, TextRenderer, TextRendererConfig};
use uuid::Uuid;
use rand::Rng;
use undo::{Edit, Record};
use stunts_engine::{
    animations::Sequence, timelines::SavedTimelineStateConfig,
};
use winit::event::{ElementState, KeyEvent, Modifiers, MouseButton, MouseScrollDelta};
use winit::dpi::{LogicalSize, PhysicalSize};
use stunts_engine::gpu_resources::GpuResources;
use editor_state::EditorState;

mod primary_canvas;
mod pipeline;
mod render_integration;
mod helpers;
mod editor_state;
mod event_handlers;
mod text_properties;

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
        opacity: String,
    },
    UpdateTextProperty {
        // text_id: String,
        property_key: String,
        property_value: String,
    },
    TogglePlay,
    ShowCaptureSources,
    StartScreenCapture { hwnd: usize },
    StopScreenCapture,
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
    fn to_animation_data(self, polygon_id: String) -> stunts_engine::animations::AnimationData {
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
            object_type: ObjectType::Polygon,
            polygon_id,
            duration: Duration::from_millis(self.duration as u64),
            start_time_ms: 0,
            properties,
            position: [0, 0],
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Stunts Native...");

    // let saved_state = load_project_state(uuid.clone().to_string())
    //     .expect("Couldn't get Saved State");

    // dummy project
    let project_id = Uuid::new_v4();
    let destination_view = "scene".to_string();
    let dummy_sequence_id = Uuid::new_v4();

    let mut dummy_sequences = Vec::new();

    dummy_sequences.push(Sequence  {
        id: dummy_sequence_id.to_string(),
        name: "Sequence 1".to_string(),
        background_fill: None,
        duration_ms: 20000,
        active_polygons: Vec::new(),
        polygon_motion_paths: Vec::new(),
        active_text_items: Vec::new(),
        active_image_items: Vec::new(),
        active_video_items: Vec::new(),
    });
    
    let saved_state = stunts_engine::saved_state::SavedState {
        id: project_id.to_string(),
        // name: "New Project".to_string(),
        sequences: dummy_sequences,
        timeline_state: SavedTimelineStateConfig {
            timeline_sequences: Vec::new(),
        },
    };

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

    editor.saved_state = Some(saved_state.clone());
    editor.project_selected = Some(project_id.clone());
    editor.current_view = destination_view.clone();

    let editor = Arc::new(Mutex::new(editor));

    // editor_state holds saved data, not active gpu data
    let cloned_editor = Arc::clone(&editor);
    let record = Arc::new(Mutex::new(Record::new()));
    let mut editor_state = editor_state::EditorState::new(cloned_editor, record.clone());
    
    let editor_state = Arc::new(Mutex::new(editor_state));

    // Create channel for communicating commands from UI to main thread
    let (command_tx, command_rx) = mpsc::channel::<Command>();
    
    // Create channel for API responses
    let (api_response_tx, api_response_rx) = mpsc::channel::<stunts_engine::animations::AnimationData>();

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
    
    // Sidebar state for property editing
    let sidebar_visible = Signal::new(false);
    let sidebar_width = 300.0;

    let motion_text = Signal::new("Motion Direction".to_string());
    let description_text = Signal::new("".to_string());
    let position_text = Signal::new("".to_string());
    let scale_text = Signal::new("".to_string());
    let opacity_text = Signal::new("".to_string());
    
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
                        // .with_signal(description_text.clone())
                        .on_change({
                            let description_text = description_text.clone();

                            move |text| {
                                description_text.set(text.to_string());
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
                        // .with_signal(position_text.clone())
                        .on_change({
                            let position_text = position_text.clone();

                            move |text| {
                                position_text.set(text.to_string());
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
                        // .with_signal(scale_text.clone())
                        .on_change({
                            let scale_text = scale_text.clone();

                            move |text| {
                                scale_text.set(text.to_string());
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
                        // .with_signal(opacity_text.clone())
                        .on_change({
                            let opacity_text = opacity_text.clone();

                            move |text| {
                                opacity_text.set(text.to_string());
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
                            let editor_for_api = editor.clone();
                            let tx = command_tx.clone();

                            move || {
                                // Send command to be processed in async context
                                tx.send(Command::SubmitMotionForm {
                                    description: description.get(),
                                    position: position.get(),
                                    scale: scale.get(),
                                    opacity: opacity.get(),
                                });
                            }
                        })
                )))
                .into_container_element()
        // ))
    );
    
    let button_square = button("Add Square")
        .with_font_size(10.0)
        .with_width(100.0)
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
        .with_width(100.0)
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
        .with_width(100.0)
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
        .with_width(100.0)
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

    let capture_button_text = if is_recording.get() {
        "Stop Recording"
    } else {
        "Screen Capture"
    }.to_string();
    
    let button_capture = button(capture_button_text)
        .with_font_size(10.0)
        .with_width(100.0)
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
    let button2 = button("Add Motion")
        .with_font_size(10.0)
        .with_width(100.0)
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
    let button3 = button("Regenerate All")
        .with_font_size(10.0)
        .with_width(100.0)
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

    // export the video
    let button4 = button("Export")
        .with_font_size(10.0)
        .with_width(100.0)
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

    // Play / pause the video
    let button5 = button("Play / Pause")
        .with_font_size(10.0)
        .with_width(100.0)
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
        .with_width(100.0)
        .with_height(20.0)
        .with_backgrounds(
            Background::Gradient(button_normal.clone()),
            Background::Gradient(button_hover.clone()),
            Background::Gradient(button_pressed.clone())
        )
        .on_click({
            let sidebar_visible = sidebar_visible.clone();
            move || {
                sidebar_visible.set(!sidebar_visible.get());
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
                            move |selected_value: String| {
                                if let Ok(hwnd) = selected_value.parse::<usize>() {
                                    println!("Selected capture source HWND: {}", hwnd);
                                    capture_sources_visible.set(false);
                                    let _ = tx.send(Command::StartScreenCapture { hwnd });
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

    let left_tools = row()
        .with_size(1100.0, 50.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Element::new_widget(Box::new(button2)))
        .with_child(Element::new_widget(Box::new(button_square)))
        .with_child(Element::new_widget(Box::new(button_text)))
        .with_child(Element::new_widget(Box::new(button_image)))
        .with_child(Element::new_widget(Box::new(button_video)))
        .with_child(Element::new_widget(Box::new(button_capture)))
        .with_child(capture_sources_dropdown.into_container_element())
        .with_child(Element::new_widget(Box::new(button3)))
        .with_child(Element::new_widget(Box::new(button4)))
        .with_child(Element::new_widget(Box::new(button_properties)));

    // let right_tools = row()
    //     .with_size(400.0, 50.0)
    //     .with_main_axis_alignment(MainAxisAlignment::Start) // horiz
    //     .with_cross_axis_alignment(CrossAxisAlignment::Start) // vert
    //     .with_child(Element::new_widget(Box::new(button3)))
    //     .with_child(Element::new_widget(Box::new(button4)));

    let toolkit = row()
        .with_size(1200.0, 50.0)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(left_tools.into_container_element());
        // .with_child(right_tools.into_container_element());

    let video_ctrls = row()
        .with_size(1200.0, 50.0)
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(button5)));

    // Create text properties widget using the new manual implementation
    let text_properties_widget = text_properties::create_text_properties_panel(
        command_tx.clone(),
        button_normal.clone(),
        button_hover.clone(),
        button_pressed.clone(),
        sidebar_width,
    );

    let sidebar_inner = column()
        .with_size(sidebar_width, 750.0)
        .with_child(text_properties_widget);

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
        .with_child(motion_form.into_container_element())
        .with_child(primary_canvas::create_render_placeholder()?)
        .with_child(video_ctrls.into_container_element());

    // Create main content area with sidebar
    let main_content = row()
        .with_size(1200.0, 800.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(property_sidebar.into_container_element())
        .with_child(main_column.into_container_element());
    
    let container = container()
        .with_size(1200.0, 800.0) 
        .with_radial_gradient(container_gradient)
        .with_padding(Padding::all(20.0))
        .with_shadow(8.0, 8.0, 15.0, Color::rgba8(0, 0, 0, 80))
        .with_child(main_content.into_container_element());
    
    let root = container.into_container_element();

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
                if let Ok(rx) = api_response_rx_for_render.try_lock() {
                    while let Ok(animation_data) = rx.try_recv() {
                        if let Ok(mut editor) = editor_for_render.try_lock() {
                            if let Ok(mut editor_state) = state_for_render.try_lock() { 
                                let sequence_data = editor.current_sequence_data.clone();
                                let last_motion_arrow_object_id = editor.last_motion_arrow_object_id.to_string();
                               
                                if let Some(ref mut saved_state) = editor.saved_state {
                                    // Clean up data
                                    let mut final_animation = animation_data.clone();
                                    final_animation.id = Uuid::new_v4().to_string();
                                    // final_animation.object_type = ObjectType::Polygon;
                                    final_animation.polygon_id = last_motion_arrow_object_id;
                                    final_animation.start_time_ms = 0;
                                    final_animation.position = [0, 0];

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
                                        
                                        println!("Animation data successfully integrated into sequence (overwrote existing)");
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Process any pending commands from the UI thread
                if let Ok(rx) = command_rx_for_render.try_lock() {
                    while let Ok(command) = rx.try_recv() {
                        if let Ok(mut editor) = editor_for_render.try_lock() {
                            if let Ok(mut editor_state) = state_for_render.try_lock() {
                                match command {
                                    Command::AddMotion => {
                                        println!("Processing add motion command from channel");
                                        editor.motion_mode = true;
                                        println!("Motion mode enabled - user can now place arrows by clicking and dragging");
                                    }
                                    Command::AddSquarePolygon => {
                                        println!("Processing add square polygon command from channel");
                                        let random_coords = helpers::utilities::get_random_coords(window_size);
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

                                        
                                        editor.add_polygon(
                                            polygon_config.clone(),
                                            polygon_config.name.clone(),
                                            polygon_config.id,
                                            dummy_sequence_id.to_string(),
                                        );

                                        editor_state.add_saved_polygon(
                                            &mut editor.saved_state.as_mut().expect("Couldn't get saved state"),
                                            dummy_sequence_id.to_string(),
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
                                            .find(|s| s.id == dummy_sequence_id.to_string())
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
                                        let random_coords = helpers::utilities::get_random_coords(window_size);
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
                                            dummy_sequence_id.to_string(),
                                        );

                                        editor_state.add_saved_text_item(
                                            &mut editor.saved_state.as_mut().expect("Couldn't get saved state"),
                                            dummy_sequence_id.to_string(),
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
                                            .find(|s| s.id == dummy_sequence_id.to_string())
                                            .expect("Couldn't get updated sequence");
                                        
                                        let sequence_cloned = updated_sequence.clone();
                                        
                                        editor.current_sequence_data = Some(sequence_cloned.clone());
                                        editor.update_motion_paths(&sequence_cloned);

                                        println!("Text item added to editor successfully: {}", text_config.id);
                                    }
                                    Command::AddImage { file_path } => {
                                        println!("Processing add image command from channel with file: {}", file_path);
                                        let random_coords = helpers::utilities::get_random_coords(window_size);
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
                                            dummy_sequence_id.to_string(),
                                        );

                                        editor_state.add_saved_image_item(
                                            &mut editor.saved_state.as_mut().expect("Couldn't get saved state"),
                                            dummy_sequence_id.to_string(),
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
                                            .find(|s| s.id == dummy_sequence_id.to_string())
                                            .expect("Couldn't get updated sequence");
                                        
                                        let sequence_cloned = updated_sequence.clone();
                                        
                                        editor.current_sequence_data = Some(sequence_cloned.clone());
                                        editor.update_motion_paths(&sequence_cloned);

                                        println!("Image item added to editor successfully: {}", image_config.id);
                                    }
                                    Command::AddVideo { file_path } => {
                                        println!("Processing add video command from channel with file: {}", file_path);
                                        let random_coords = helpers::utilities::get_random_coords(window_size);
                                        let new_id = Uuid::new_v4();
                                        
                                        // Extract filename for a better name
                                        let filename = std::path::Path::new(&file_path)
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("Video");

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
                                            mouse_path: None,
                                        };

                                        let window_size = WindowSize {
                                            width: window_size.width,
                                            height: window_size.height,
                                        };

                                        editor.add_video_item(
                                            &window_size,
                                            device,
                                            queue,
                                            video_config.clone(),
                                            &Path::new(&file_path.clone()),
                                            new_id,
                                            dummy_sequence_id.to_string(),
                                            None, // stored_mouse_positions
                                            None, // stored_source_data
                                        );

                                        let source_duration_ms = editor
                                            .video_items
                                            .last()
                                            .expect("Couldn't get latest video")
                                            .source_duration_ms
                                            .clone();

                                        editor_state.add_saved_video_item(
                                            &mut editor.saved_state.as_mut().expect("Couldn't get saved state"),
                                            dummy_sequence_id.to_string(),
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
                                                mouse_path: None,
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
                                            .find(|s| s.id == dummy_sequence_id.to_string())
                                            .expect("Couldn't get updated sequence");
                                        
                                        let sequence_cloned = updated_sequence.clone();
                                        
                                        editor.current_sequence_data = Some(sequence_cloned.clone());
                                        editor.update_motion_paths(&sequence_cloned);

                                        println!("Video item added to editor successfully: {}", video_config.id);
                                    }
                                    Command::SubmitMotionForm { description, position, scale, opacity } => {
                                        println!("Processing motion form submission from channel");

                                        // Reset canvas hidden state
                                        // let mut editor_lock = editor_for_render.lock().unwrap();
                                        editor.canvas_hidden = false;
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
                                            "arrow_positions": arrow_positions.map(|(p1, p2)| serde_json::json!({"startX": p1.x, "startY": p1.y, "endX": p2.x, "endY": p2.y})),
                                            "object_dimensions": object_dimensions.map(|(w, h)| serde_json::json!({"width": w, "height": h}))
                                        });
                                        
                                        // Clone the response sender for the async block
                                        let response_sender = api_response_tx_for_render.clone();
                                        
                                        // Get the polygon_id before the async block to avoid Send issues
                                        let polygon_id = editor.last_motion_arrow_object_id.to_string();

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
                                                                        let animation_data = api_data.to_animation_data(polygon_id);
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
                                                        value: source.hwnd.to_string(),
                                                    }
                                                }).collect());
                                            }
                                            Err(e) => {
                                                println!("Failed to get capture sources: {}", e);
                                            }
                                        }
                                    }
                                    Command::StartScreenCapture { hwnd } => {
                                        println!("Processing start screen capture command for HWND: {}", hwnd);

                                        

                                        // Start recording (this will handle HWND conversion internally)
                                        // Note: start_video_capture expects (hwnd, width, height, project_id)
                                        // TODO: We need to get the window dimensions for the selected HWND
                                        match editor.st_capture.start_video_capture(hwnd, 1920, 1080, project_id.to_string()) {
                                            Ok(_) => {
                                                println!("Screen capture started successfully");
                                                is_recording.set(true);
                                                capture_sources_visible.set(false);
                                            }
                                            Err(e) => {
                                                println!("Failed to start screen capture: {}", e);
                                            }
                                        }
                                    }
                                    Command::StopScreenCapture => {
                                        println!("Processing stop screen capture command");

                                        

                                        match editor.st_capture.stop_video_capture(project_id.to_string()) {
                                            Ok((video_path, mouse_data_path)) => {
                                                println!("Screen capture stopped successfully");
                                                println!("Video saved to: {:?}", video_path);
                                                println!("Mouse data saved to: {:?}", mouse_data_path);

                                                is_recording.set(false);

                                                // Automatically add the captured video to the current sequence
                                                // video_path is already a String, not a PathBuf
                                                // let tx_clone = command_tx.clone();
                                                // let _ = tx_clone.send(Command::AddVideo {
                                                //     file_path: video_path.clone()
                                                // });
                                            }
                                            Err(e) => {
                                                println!("Failed to stop screen capture: {}", e);
                                                is_recording.set(false);
                                            }
                                        }
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