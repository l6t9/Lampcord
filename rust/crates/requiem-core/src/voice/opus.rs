use audiopus::coder::{Decoder, Encoder};
use audiopus::{Application, Bitrate, Channels, SampleRate};

pub const SAMPLE_RATE: u32 = 48_000;

pub const FRAME_SAMPLES: usize = 960;

pub const MAX_PACKET: usize = 4000;

pub struct OpusEncoder {
    enc: Encoder,
}

impl OpusEncoder {
    pub fn new() -> Result<Self, audiopus::Error> {
        let mut enc = Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip)?;

        enc.set_bitrate(Bitrate::BitsPerSecond(64_000))?;
        Ok(Self { enc })
    }

    pub fn encode(
        &self,
        input: &[f32],
        output: &mut [u8; MAX_PACKET],
    ) -> Result<usize, audiopus::Error> {
        debug_assert_eq!(input.len(), FRAME_SAMPLES);
        self.enc.encode_float(input, output)
    }
}

pub struct OpusDecoder {
    dec: Decoder,
}

impl OpusDecoder {
    pub fn new() -> Result<Self, audiopus::Error> {
        Ok(Self {
            dec: Decoder::new(SampleRate::Hz48000, Channels::Stereo)?,
        })
    }

    pub fn decode(&mut self, packet: &[u8], output: &mut [f32]) -> Result<usize, audiopus::Error> {
        let packet = audiopus::packet::Packet::try_from(packet)?;
        let signals = audiopus::MutSignals::try_from(output)?;
        self.dec.decode_float(Some(packet), signals, false)
    }

    pub fn decode_lost(&mut self, output: &mut [f32]) -> Result<usize, audiopus::Error> {
        let signals = audiopus::MutSignals::try_from(output)?;
        self.dec.decode_float(None, signals, false)
    }
}

pub const STEREO_FRAME_SAMPLES: usize = FRAME_SAMPLES * 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_silence() {
        let enc = OpusEncoder::new().unwrap();
        let mut dec = OpusDecoder::new().unwrap();

        let silence = vec![0.0f32; FRAME_SAMPLES];
        let mut packet = [0u8; MAX_PACKET];
        let n = enc.encode(&silence, &mut packet).unwrap();
        assert!(n > 0 && n < MAX_PACKET);

        let mut out = vec![0.0f32; STEREO_FRAME_SAMPLES];
        let samples = dec.decode(&packet[..n], &mut out).unwrap();
        assert_eq!(samples, FRAME_SAMPLES);
    }
}
