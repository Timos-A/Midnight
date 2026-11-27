use cpal::traits::{DeviceTrait, HostTrait};

pub fn output_devices(host: &cpal::Host) -> Vec<String> {
    host.output_devices()
        .into_iter()
        .flatten()
        .filter_map(|device| {
            device
                .description()
                .ok()
                .map(|description| description.name().to_owned())
        })
        .collect()
}

pub fn default_output_index(host: &cpal::Host, devices: &[String]) -> Option<usize> {
    let default_name = host.default_output_device().and_then(|device| {
        device
            .description()
            .ok()
            .map(|description| description.name().to_owned())
    });
    default_name
        .and_then(|name| devices.iter().position(|device| *device == name))
        .or_else(|| (!devices.is_empty()).then_some(0))
}

pub fn find_output_device(host: &cpal::Host, selected_name: &str) -> Result<cpal::Device, String> {
    host.output_devices()
        .map_err(|error| format!("Could not list audio output devices: {error}"))?
        .find(|device| {
            device
                .description()
                .ok()
                .is_some_and(|description| description.name() == selected_name)
        })
        .ok_or_else(|| format!("Selected audio output is unavailable: {selected_name}"))
}
