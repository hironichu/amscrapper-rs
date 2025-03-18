use std::collections::HashMap;
use std::sync::RwLock;

use crate::music::{AMusicSongInfo, AMusicState, AMusicTimeInfo};
use anyhow::Result;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;

use uiautomation::UIAutomation;
use uiautomation::UIElement;
use uiautomation::patterns::UIRangeValuePattern;
use uiautomation::types::PropertyConditionFlags;
use uiautomation::variants::Variant;

impl Drop for AMusicScraper {
    fn drop(&mut self) {
        self.composer_performer_regex = Regex::new("").unwrap();
        self.automation = UIAutomation::new().unwrap();
        self.window = None;
        self.amsongpanel = None;
        self.amsong_field_panel = None;
    }
}
#[derive(Debug, Clone)]
pub struct AMusicScraper {
    composer_performer_regex: Regex,
    automation: UIAutomation,
    window: Option<UIElement>,
    amsongpanel: Option<UIElement>,
    amsong_field_panel: Option<UIElement>,
}

unsafe impl Send for AMusicScraper {}
unsafe impl Sync for AMusicScraper {}

impl AMusicScraper {
    pub fn new(automation: UIAutomation, window: UIElement) -> Result<Self, anyhow::Error> {
        let mut _self = Self {
            composer_performer_regex: Regex::new(r"By (.+) \u2014 (.+) \u2014 (.+)").unwrap(),
            automation,
            window: Some(window),
            amsongpanel: None,
            amsong_field_panel: None,
        };
        _self.init_elements()?;
        Ok(_self)
    }

    pub fn init_elements(&mut self) -> Result<(), anyhow::Error> {
        if self.window.is_none() {
            return Err(anyhow::Error::msg("No window defined"));
        }
        let window = self.window.as_ref().unwrap();

        let automation = self.automation.clone();
        let amsongpanel = window.find_first(
            uiautomation::types::TreeScope::Descendants,
            &automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("TransportBar"),
                    None,
                )
                .unwrap(),
        );

        if amsongpanel.is_err() {
            return Err(anyhow::Error::msg("No song panel"));
        }
        let amsongpanel = amsongpanel.unwrap();

        let amsong_field_panel = amsongpanel.find_first(
            uiautomation::types::TreeScope::Children,
            &automation
                .create_property_condition(
                    uiautomation::types::UIProperty::AutomationId,
                    Variant::from("LCD"),
                    None,
                )
                .unwrap(),
        );

        if amsong_field_panel.is_err() {
            return Err(anyhow::Error::msg("No song field panel"));
        }
        let amsong_field_panel = amsong_field_panel.unwrap();
        self.amsong_field_panel = Some(amsong_field_panel);
        self.amsongpanel = Some(amsongpanel);
        Ok(())
    }
}
