use amscrapper_rs::utils::{self, APPLE_MUSIC, SPOTIFY_FREE};
use anyhow::Result;
use uiautomation::{UIAutomation, variants::Variant};
fn main() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    compile_error!("Only Windows is supported");

    let automation = UIAutomation::new().unwrap().clone();

    let window = utils::grab_window(SPOTIFY_FREE, &automation, false);
    if window.is_none() {
        println!("No window found");
        return Ok(());
    }
    let window = window.unwrap();
    // let window = self.window.as_ref().unwrap();
    let matcherdebug = automation
        .create_matcher()
        .from(window)
        .depth(2000)
        .find_all()
        .unwrap();

    for i in matcherdebug {
        let name = i.get_name().unwrap();
        let classname = i.get_classname().unwrap();
        let automation_id = i.get_automation_id().unwrap();

        //check if name contains Engines Ready, it may just CONTAIN it, not be equal to it
        if name.contains("Timing") {
            println!("Found Engines Ready");
            println!(
                "Name : {:?}      | Class : {}         | AutomationID:  {}",
                i.get_name().unwrap(),
                i.get_classname().unwrap(),
                i.get_automation_id().unwrap()
            );
        }
    }
    // // let automation = self.automation.clone();
    // let amsongpanel = window.find_first(
    //     uiautomation::types::TreeScope::Descendants,
    //     &automation
    //         .create_property_condition(
    //             uiautomation::types::UIProperty::AutomationId,
    //             Variant::from("TransportBar"),
    //             None,
    //         )
    //         .unwrap(),
    // );

    // if amsongpanel.is_err() {
    //     return Err(anyhow::Error::msg("No song panel"));
    // }
    // let amsongpanel = amsongpanel.unwrap();

    // let amsong_field_panel = amsongpanel.find_first(
    //     uiautomation::types::TreeScope::Children,
    //     &automation
    //         .create_property_condition(
    //             uiautomation::types::UIProperty::AutomationId,
    //             Variant::from("LCD"),
    //             None,
    //         )
    //         .unwrap(),
    // );

    // if amsong_field_panel.is_err() {
    //     return Err(anyhow::Error::msg("No song field panel"));
    // }
    // let amsong_field_panel = amsong_field_panel.unwrap();}
    Ok(())
}
