use iced::widget::{bottom_right, button, column, container, scrollable, stack, text};
use iced::{Element, Fill};
use serde::Deserialize;

use crate::theme;

/// The form fields for a Root Ventures application
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Form {
    pub name: String,
    pub email: String,
    pub linkedin: String,
    pub github: String,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolInput {
    pub field: String,
    pub action: String,
    #[serde(default)]
    pub value: String,
}

impl Form {
    /// Returns true when the form is empty
    pub fn is_empty(&self) -> bool {
        self == &Form::default()
    }

    /// Returns true when required fields are filled
    pub fn is_ready(&self) -> bool {
        !self.name.trim().is_empty() && !self.email.trim().is_empty()
    }

    /// Apply a tool invocation, return a result string for the tool_result
    pub fn apply(&mut self, input: &ToolInput) -> String {
        match input.action.as_str() {
            "write" => {
                match input.field.as_str() {
                    "name" => self.name = input.value.clone(),
                    "email" => self.email = input.value.clone(),
                    "linkedin" => self.linkedin = input.value.clone(),
                    "github" => self.github = input.value.clone(),
                    "notes" => self.notes = input.value.clone(),
                    _ => return format!("Unknown field: {}", input.field),
                }
                format!("Set {} to {:?}", input.field, input.value)
            }
            "read" => {
                if input.field == "form" {
                    format!(
                        "name: {:?}\nemail: {:?}\nlinkedin: {:?}\ngithub: {:?}\nnotes: {:?}",
                        self.name, self.email, self.linkedin, self.github, self.notes
                    )
                } else {
                    match input.field.as_str() {
                        "name" => format!("{:?}", self.name),
                        "email" => format!("{:?}", self.email),
                        "linkedin" => format!("{:?}", self.linkedin),
                        "github" => format!("{:?}", self.github),
                        "notes" => format!("{:?}", self.notes),
                        _ => format!("Unknown field: {}", input.field),
                    }
                }
            }
            "clear" => {
                if input.field == "form" {
                    *self = Form::default();
                    "Form cleared".to_string()
                } else {
                    match input.field.as_str() {
                        "name" => self.name.clear(),
                        "email" => self.email.clear(),
                        "linkedin" => self.linkedin.clear(),
                        "github" => self.github.clear(),
                        "notes" => self.notes.clear(),
                        _ => return format!("Unknown field: {}", input.field),
                    }
                    format!("Cleared {}", input.field)
                }
            }
            "submit" => {
                if self.is_ready() {
                    "Form submitted".to_string()
                } else {
                    "Cannot submit: name and email are required".to_string()
                }
            }
            _ => format!("Unknown action: {}", input.action),
        }
    }

    /// Returns the tool definition JSON for the Claude API request
    pub fn tool_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "update_form",
            "description": "Update the job application form. Use this to write, read, or clear form fields. Fields: name (required), email (required), linkedin, github, notes. Use field 'form' with action 'clear' to reset all fields, 'read' to see all values, or 'submit' to submit the application.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "field": {
                        "type": "string",
                        "enum": ["name", "email", "linkedin", "github", "notes", "form"],
                        "description": "Which field to act on. 'form' targets the whole form."
                    },
                    "action": {
                        "type": "string",
                        "enum": ["write", "read", "clear", "submit"],
                        "description": "What to do: write a value, read the current value, clear it, or submit."
                    },
                    "value": {
                        "type": "string",
                        "description": "The value to write (only used with action 'write')."
                    }
                },
                "required": ["field", "action"]
            }
        })
    }

    /// Render the form as a sidebar Element
    pub fn view<'a, Message: Clone + 'a>(
        &self,
        on_submit: Option<Message>,
    ) -> Element<'a, Message> {
        let dim = iced::Color::from_rgb(0.5, 0.5, 0.5);

        let field = |label: &'static str, value: String| -> Element<'a, Message> {
            column![
                text(label).size(12).color(dim),
                if value.is_empty() {
                    text("—").size(16)
                } else {
                    text(value).size(16)
                },
            ]
            .spacing(2)
            .into()
        };

        let submit = button("Submit").on_press_maybe(on_submit);

        let fields = scrollable(
            column![
                text("Application").size(20),
                field("Name *", self.name.clone()),
                field("Email *", self.email.clone()),
                field("LinkedIn", self.linkedin.clone()),
                field("GitHub", self.github.clone()),
                field("Notes", self.notes.clone()),
            ]
            .spacing(12)
            .padding(16),
        );

        container(stack![bottom_right(submit).padding(16), fields])
            .style(theme::sidebar)
            .width(Fill)
            .height(Fill)
            .into()
    }
}
