use gui_core::{Element, widgets::*};
use gui_core::widgets::container::{Padding, Background};
use gui_core::widgets::text::text_signal;
use gui_reactive::Signal;
use vello::peniko::{Color, Gradient};
use std::sync::{Arc, Mutex, mpsc};
use crate::Command;
use stunts_engine::editor::rgb_to_wgpu;

pub const THEME_COLORS: [[&str; 5]; 10] = [
    ["#FFE4E1", "#FF6B6B", "#FF0000", "#B22222", "#8B0000"], // red
    ["#FFECD9", "#FFB347", "#FF8C00", "#D95E00", "#993D00"], // orange
    ["#FFFACD", "#FFE66D", "#FFD700", "#DAA520", "#B8860B"], // yellow
    ["#E8F5E9", "#7CB342", "#2E7D32", "#1B5E20", "#0A3D0A"], // green
    ["#E3F2FD", "#64B5F6", "#1E88E5", "#1565C0", "#0D47A1"], // blue
    ["#F3E5F5", "#AB47BC", "#8E24AA", "#6A1B9A", "#4A148C"], // purple
    ["#FCE4EC", "#F06292", "#E91E63", "#C2185B", "#880E4F"], // pink
    ["#E0F2F1", "#4DB6AC", "#00897B", "#00695C", "#004D40"], // teal
    ["#EFEBE9", "#A1887F", "#795548", "#5D4037", "#3E2723"], // brown
    ["#F5F5F5", "#BDBDBD", "#757575", "#424242", "#212121"], // gray
];

pub fn create_themes_sidebar_panel(
    command_tx: mpsc::Sender<Command>,
    sidebar_width: f32,
) -> Element {

    // 50 color / text combinations (style portion of format)
    // background_color_index, text_length, font_family_index, font_size, font_color_index
    let themes = [
        [0.0, 120.0, 12.0, 24.0, 0.4],
        [1.2, 80.0, 25.0, 32.0, 1.0],
        [2.1, 150.0, 37.0, 18.0, 2.3],
        [3.3, 200.0, 45.0, 20.0, 3.1],
        [4.4, 100.0, 50.0, 28.0, 4.0],
        [5.2, 90.0, 55.0, 22.0, 5.1],
        [6.0, 130.0, 10.0, 26.0, 6.3],
        [7.2, 110.0, 30.0, 16.0, 7.4],
        [8.1, 140.0, 40.0, 20.0, 8.3],
        [9.3, 180.0, 5.0, 18.0, 9.1],
        [0.1, 95.0, 18.0, 30.0, 0.3],
        [1.3, 110.0, 22.0, 20.0, 1.2],
        [2.2, 130.0, 35.0, 22.0, 2.4],
        [3.0, 160.0, 48.0, 18.0, 3.2],
        [4.1, 75.0, 7.0, 28.0, 4.3],
        [5.4, 140.0, 53.0, 24.0, 5.0],
        [6.2, 100.0, 14.0, 26.0, 6.1],
        [7.1, 120.0, 29.0, 20.0, 7.3],
        [8.2, 150.0, 42.0, 18.0, 8.4],
        [9.0, 200.0, 3.0, 16.0, 9.2],
        [0.3, 85.0, 20.0, 32.0, 0.2],
        [1.4, 105.0, 26.0, 24.0, 1.1],
        [2.0, 115.0, 38.0, 20.0, 2.3],
        [3.2, 170.0, 47.0, 18.0, 3.4],
        [4.2, 90.0, 9.0, 30.0, 4.1],
        [5.1, 125.0, 54.0, 22.0, 5.3],
        [6.3, 135.0, 16.0, 24.0, 6.2],
        [7.0, 145.0, 31.0, 18.0, 7.4],
        [8.3, 155.0, 43.0, 20.0, 8.1],
        [9.4, 180.0, 6.0, 16.0, 9.0],
        [0.4, 100.0, 23.0, 28.0, 0.1],
        [1.0, 115.0, 27.0, 22.0, 1.3],
        [2.3, 140.0, 39.0, 20.0, 2.2],
        [3.1, 160.0, 46.0, 18.0, 3.0],
        [4.3, 80.0, 8.0, 32.0, 4.2],
        [5.0, 130.0, 55.0, 24.0, 5.4],
        [6.1, 95.0, 15.0, 26.0, 6.4],
        [7.3, 110.0, 32.0, 20.0, 7.2],
        [8.4, 165.0, 44.0, 18.0, 8.0],
        [9.2, 190.0, 4.0, 16.0, 9.3],
    ];

    // Create theme buttons
    let mut theme_buttons = Vec::new();
    
    for (index, theme) in themes.iter().enumerate() {
        let background_color_row = (theme[0] as f64).trunc() as usize;
        let background_color_column = ((theme[0] as f64).fract() * 10.0) as usize;
        let background_color_str = THEME_COLORS[background_color_row][background_color_column];
        
        let text_color_row = (theme[4] as f64).trunc() as usize; 
        let text_color_column = ((theme[4] as f64).fract() * 10.0) as usize;
        let text_color_str = THEME_COLORS[text_color_row][text_color_column];
        
        // Parse hex colors
        let bg_color = parse_hex_color(background_color_str);
        let text_color = parse_hex_color(text_color_str);
        
        let theme_clone = *theme;
        let tx = command_tx.clone();
        
        let theme_button = button("Apply Theme")
            .with_size(120.0, 50.0)
            .with_background(Background::Color(bg_color))
            // .with_text_color(text_color)
            .on_click(move || {
                let _ = tx.send(Command::ApplyTheme { 
                    theme: theme_clone 
                });
            });
            
        theme_buttons.push(Element::new_widget(Box::new(theme_button)));
    }

    // Create a grid layout for theme buttons (2 columns to fit in sidebar)
    // let mut theme_rows = Vec::new();
    // for chunk in theme_buttons.chunks(2) {
    //     let mut theme_row = row()
    //         .with_size(sidebar_width - 20.0, 60.0)
    //         .with_main_axis_alignment(MainAxisAlignment::SpaceAround)
    //         .with_cross_axis_alignment(CrossAxisAlignment::Center);
            
    //     for button in chunk {
    //         theme_row = theme_row.with_child(button.clone());
    //     }
        
    //     theme_rows.push(theme_row.into_container_element());
    // }

    // Section header
    let themes_header = Element::new_widget(Box::new(
        text("Themes")
            .with_font_size(14.0)
            .with_color(Color::rgba8(255, 255, 255, 255))
    ));

    // Main column layout
    let mut main_column = column()
        .with_size(sidebar_width, 600.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(themes_header);
        
    for theme_row in theme_buttons {
        main_column = main_column.with_child(theme_row);
    }

    main_column.into_container_element()
}

fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color::rgba8(r, g, b, 255)
    } else {
        Color::rgba8(128, 128, 128, 255) // fallback gray
    }
}