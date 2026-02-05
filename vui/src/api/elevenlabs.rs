use serde::Deserialize;

pub struct Client {
    api_key: String,
    http: reqwest::Client,
    voice_id: String,
}

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

impl Client {
    pub fn new(api_key: String) -> Self {
        let http = reqwest::Client::new();
        Self {
            api_key,
            http,
            voice_id: "21m00Tcm4TlvDq8ikWAM".to_string(), // Rachel
        }
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>, String> {
        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}",
            self.voice_id
        );

        let body = serde_json::json!({
            "text": text,
            "model_id": "eleven_turbo_v2_5",
            "voice_settings": {
                "stability": 0.5,
                "similarity_boost": 0.75
            }
        });

        let res = self
            .http
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "audio/mpeg")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!("ElevenLabs TTS error: {}", error_text));
        }

        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        Ok(bytes.to_vec())
    }

    pub async fn transcribe(
        &self,
        audio_samples: &[f32],
        sample_rate: u32,
    ) -> Result<String, String> {
        let wav_data = encode_wav(audio_samples, sample_rate);

        let part = reqwest::multipart::Part::bytes(wav_data)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| e.to_string())?;

        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model_id", "scribe_v1")
            .text("language_code", "en");

        let res = self
            .http
            .post("https://api.elevenlabs.io/v1/speech-to-text")
            .header("xi-api-key", &self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!("ElevenLabs STT error: {}", error_text));
        }

        let response: TranscriptionResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(response.text)
    }
}

fn encode_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample) / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = (samples.len() * 2) as u32;

    let mut wav = Vec::with_capacity(44 + samples.len() * 2);

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for sample in samples {
        let s = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        wav.extend_from_slice(&s.to_le_bytes());
    }

    wav
}
