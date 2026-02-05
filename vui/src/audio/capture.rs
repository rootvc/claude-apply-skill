use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use std::sync::mpsc;

pub struct Capture {
    #[allow(dead_code)]
    stream: cpal::Stream,
    receiver: mpsc::Receiver<Vec<f32>>,
}

impl Capture {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let config = device
            .default_input_config()
            .map_err(|e| e.to_string())?;

        let (sender, receiver) = mpsc::channel();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), sender),
            cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), sender),
            cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), sender),
            _ => return Err("Unsupported sample format".to_string()),
        }
        .map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;

        Ok(Self { stream, receiver })
    }

    pub fn try_recv(&self) -> Option<Vec<f32>> {
        self.receiver.try_recv().ok()
    }

    pub fn sample_rate() -> u32 {
        let host = cpal::default_host();
        host.default_input_device()
            .and_then(|d| d.default_input_config().ok())
            .map(|c| c.sample_rate())
            .unwrap_or(16000)
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sender: mpsc::Sender<Vec<f32>>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: cpal::Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let samples: Vec<f32> = data.iter().map(|s| Sample::from_sample(*s)).collect();
            let _ = sender.send(samples);
        },
        |err| eprintln!("Audio capture error: {}", err),
        None,
    )
}
