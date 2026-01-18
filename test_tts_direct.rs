use vibespeak::infrastructure::adapters::TtsAdapter;
use vibespeak::domain::services::TextToSpeechService;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing TTS adapter directly...");

    let adapter = TtsAdapter::new()?;
    println!("✓ TTS adapter initialized");

    // Test in a blocking context using tokio
    let rt = tokio::runtime::Runtime::new()?;
    let samples = rt.block_on(adapter.synthesize("Hello, this is a test of the Piper TTS system", None))?;
    println!("✓ TTS synthesis successful: {} samples generated", samples.len());

    // Check if samples look like real audio data
    if samples.is_empty() {
        println!("✗ ERROR: Empty samples returned!");
        return Err("Empty samples".into());
    }

    // Check first few samples
    println!("First 5 samples: [{}, {}, {}, {}, {}]", samples[0], samples[1], samples[2], samples[3], samples[4]);

    // Check if samples vary (not all zeros)
    let has_variation = samples.windows(2).any(|w| w[0] != w[1]);
    if !has_variation {
        println!("✗ WARNING: Samples appear to be constant (possibly dummy data)");
    } else {
        println!("✓ Samples show variation (likely real audio data)");
    }

    // Check for reasonable audio range (-32768 to 32767 for 16-bit)
    let max_sample = samples.iter().map(|&s| s.abs()).max().unwrap_or(0);
    if max_sample > 0 && max_sample <= 32767 {
        println!("✓ Sample values are in valid 16-bit audio range");
    } else {
        println!("✗ WARNING: Sample values outside normal audio range: max={}", max_sample);
    }

    println!("✓ TTS adapter test completed successfully!");
    Ok(())
}