use gui_core::{Element, widgets::*};
use gui_core::widgets::container::{Padding, Background};
use gui_core::widgets::text::text_signal;
use gui_core::widgets::dropdown::{DropdownOption, dropdown};
use gui_reactive::Signal;
use vello::peniko::{Color, Gradient};
use std::sync::mpsc;
use crate::Command;

pub fn create_text_properties_panel(
    command_tx: mpsc::Sender<Command>,
    button_normal: Gradient,
    button_hover: Gradient,
    button_pressed: Gradient,
    sidebar_width: f32,
) -> Element {
    // Create signals for property values
    let text_content = Signal::new("Sample Text".to_string());
    let font_size = Signal::new("24".to_string());
    let position_x = Signal::new("0".to_string());
    let position_y = Signal::new("0".to_string());
    let width = Signal::new("200".to_string());
    let height = Signal::new("50".to_string());
    
    // Font family dropdown options
    let font_options = vec![
        DropdownOption::new("Actor", "Actor"),
        DropdownOption::new("Aladin", "Aladin"),
        DropdownOption::new("Aleo", "Aleo"),
        DropdownOption::new("Amiko", "Amiko"),
        DropdownOption::new("Ballet", "Ballet"),
        DropdownOption::new("Basic", "Basic"),
        DropdownOption::new("Bungee", "Bungee"),
        DropdownOption::new("Caramel", "Caramel"),
        DropdownOption::new("Cherish", "Cherish"),
        DropdownOption::new("Coda", "Coda"),
    ];
    
    let selected_font = Signal::new("Aleo".to_string());

    // Text Properties Section
    let text_content_row = row()
        .with_size(sidebar_width - 20.0, 35.0)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(
            text("Text:")
                .with_font_size(12.0)
                .with_color(Color::rgba8(200, 200, 200, 255))
        )))
        .with_child(Element::new_widget(Box::new(
            input()
                .with_width(150.0)
                .with_height(25.0)
                .with_text(text_content.get().as_str())
                .on_change({
                    let text_content = text_content.clone();
                    let tx = command_tx.clone();
                    move |text| {
                        text_content.set(text.to_string());
                        let _ = tx.send(Command::UpdateTextProperty {
                            property_key: "text_content".to_string(),
                            property_value: text.to_string(),
                        });
                    }
                })
        )));

    let font_family_row = row()
        .with_size(sidebar_width - 20.0, 35.0)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(
            text("Font:")
                .with_font_size(12.0)
                .with_color(Color::rgba8(200, 200, 200, 255))
        )))
        .with_child(Element::new_widget(Box::new(
            dropdown()
                .with_options(font_options)
                .with_selected_value(selected_font.get())
                .with_size(150.0, 25.0)
                .on_selection_changed({
                    let selected_font = selected_font.clone();
                    let tx = command_tx.clone();
                    move |selection| {
                        selected_font.set(selection.clone());
                        let _ = tx.send(Command::UpdateTextProperty {
                            property_key: "font_family".to_string(),
                            property_value: selection,
                        });
                    }
                })
        )));

    let font_size_row = row()
        .with_size(sidebar_width - 20.0, 35.0)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(
            text("Size:")
                .with_font_size(12.0)
                .with_color(Color::rgba8(200, 200, 200, 255))
        )))
        .with_child(Element::new_widget(Box::new(
            input()
                .with_width(150.0)
                .with_height(25.0)
                .with_text(font_size.get().as_str())
                .on_change({
                    let font_size = font_size.clone();
                    let tx = command_tx.clone();
                    move |text| {
                        font_size.set(text.to_string());
                        let _ = tx.send(Command::UpdateTextProperty {
                            property_key: "font_size".to_string(),
                            property_value: text.to_string(),
                        });
                    }
                })
        )));

    // Transform Properties Section
    let position_x_row = row()
        .with_size(sidebar_width - 20.0, 35.0)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(
            text("X:")
                .with_font_size(12.0)
                .with_color(Color::rgba8(200, 200, 200, 255))
        )))
        .with_child(Element::new_widget(Box::new(
            input()
                .with_width(150.0)
                .with_height(25.0)
                .with_text(position_x.get().as_str())
                .on_change({
                    let position_x = position_x.clone();
                    let tx = command_tx.clone();
                    move |text| {
                        position_x.set(text.to_string());
                        let _ = tx.send(Command::UpdateTextProperty {
                            property_key: "position_x".to_string(),
                            property_value: text.to_string(),
                        });
                    }
                })
        )));

    let position_y_row = row()
        .with_size(sidebar_width - 20.0, 35.0)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(
            text("Y:")
                .with_font_size(12.0)
                .with_color(Color::rgba8(200, 200, 200, 255))
        )))
        .with_child(Element::new_widget(Box::new(
            input()
                .with_width(150.0)
                .with_height(25.0)
                .with_text(position_y.get().as_str())
                .on_change({
                    let position_y = position_y.clone();
                    let tx = command_tx.clone();
                    move |text| {
                        position_y.set(text.to_string());
                        let _ = tx.send(Command::UpdateTextProperty {
                            property_key: "position_y".to_string(),
                            property_value: text.to_string(),
                        });
                    }
                })
        )));

    let width_row = row()
        .with_size(sidebar_width - 20.0, 35.0)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(
            text("Width:")
                .with_font_size(12.0)
                .with_color(Color::rgba8(200, 200, 200, 255))
        )))
        .with_child(Element::new_widget(Box::new(
            input()
                .with_width(150.0)
                .with_height(25.0)
                .with_text(width.get().as_str())
                .on_change({
                    let width = width.clone();
                    let tx = command_tx.clone();
                    move |text| {
                        width.set(text.to_string());
                        let _ = tx.send(Command::UpdateTextProperty {
                            property_key: "width".to_string(),
                            property_value: text.to_string(),
                        });
                    }
                })
        )));

    let height_row = row()
        .with_size(sidebar_width - 20.0, 35.0)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(Element::new_widget(Box::new(
            text("Height:")
                .with_font_size(12.0)
                .with_color(Color::rgba8(200, 200, 200, 255))
        )))
        .with_child(Element::new_widget(Box::new(
            input()
                .with_width(150.0)
                .with_height(25.0)
                .with_text(height.get().as_str())
                .on_change({
                    let height = height.clone();
                    let tx = command_tx.clone();
                    move |text| {
                        height.set(text.to_string());
                        let _ = tx.send(Command::UpdateTextProperty {
                            property_key: "height".to_string(),
                            property_value: text.to_string(),
                        });
                    }
                })
        )));

    // Section headers
    let text_properties_header = Element::new_widget(Box::new(
        text("Text Properties")
            .with_font_size(14.0)
            .with_color(Color::rgba8(255, 255, 255, 255))
    ));

    let transform_properties_header = Element::new_widget(Box::new(
        text("Transform")
            .with_font_size(14.0)
            .with_color(Color::rgba8(255, 255, 255, 255))
    ));

    // Main column layout
    column()
        .with_size(sidebar_width, 400.0)
        .with_main_axis_alignment(MainAxisAlignment::Start)
        .with_cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(text_properties_header)
        .with_child(text_content_row.into_container_element())
        .with_child(font_family_row.into_container_element())
        .with_child(font_size_row.into_container_element())
        .with_child(Element::new_widget(Box::new(
            text("")
                .with_font_size(8.0) // Spacer
        )))
        .with_child(transform_properties_header)
        .with_child(position_x_row.into_container_element())
        .with_child(position_y_row.into_container_element())
        .with_child(width_row.into_container_element())
        .with_child(height_row.into_container_element())
        .into_container_element()
}