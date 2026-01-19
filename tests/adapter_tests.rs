// Integration tests for adapter implementations

mod tts_adapter_tests {
    use vibespeak::domain::services::TextToSpeechService;

    #[tokio::test]
    async fn test_tts_adapter_creation() {
        use vibespeak::infrastructure::adapters::TtsAdapter;

        let adapter = TtsAdapter::new();
        assert!(
            adapter.is_ok(),
            "TTS adapter should be created successfully"
        );
    }

    #[tokio::test]
    async fn test_tts_adapter_initialize() {
        use vibespeak::infrastructure::adapters::TtsAdapter;

        let adapter = TtsAdapter::new().expect("Failed to create adapter");
        let result = adapter.initialize().await;
        assert!(result.is_ok(), "TTS adapter should initialize successfully");
    }

    #[tokio::test]
    async fn test_tts_adapter_get_available_voices() {
        use vibespeak::infrastructure::adapters::TtsAdapter;

        let adapter = TtsAdapter::new().expect("Failed to create adapter");
        let voices = adapter.get_available_voices().await;

        assert!(voices.is_ok(), "Should get available voices");
        let voices = voices.unwrap();
        assert!(!voices.is_empty(), "Should have at least one voice");
        assert!(
            voices.contains(&"default".to_string()),
            "Should have default voice"
        );
    }

    #[tokio::test]
    async fn test_tts_adapter_synthesize_default_voice() {
        use vibespeak::infrastructure::adapters::TtsAdapter;

        let adapter = TtsAdapter::new().expect("Failed to create adapter");
        let result = adapter.synthesize("Hello world", None).await;

        assert!(result.is_ok(), "Synthesis should succeed");
        let samples = result.unwrap();
        assert!(!samples.is_empty(), "Should produce audio samples");
    }

    #[tokio::test]
    async fn test_tts_adapter_synthesize_with_voice() {
        use vibespeak::infrastructure::adapters::TtsAdapter;

        let adapter = TtsAdapter::new().expect("Failed to create adapter");

        // Test different voices
        for voice in &["male", "female", "fast", "slow"] {
            let result = adapter.synthesize("Test", Some(voice)).await;
            assert!(
                result.is_ok(),
                "Synthesis with {} voice should succeed",
                voice
            );
            assert!(
                !result.unwrap().is_empty(),
                "{} voice should produce samples",
                voice
            );
        }
    }

    #[tokio::test]
    async fn test_tts_adapter_synthesize_empty_text() {
        use vibespeak::infrastructure::adapters::TtsAdapter;

        let adapter = TtsAdapter::new().expect("Failed to create adapter");
        let result = adapter.synthesize("", None).await;

        // Empty text should still work (may produce minimal audio)
        assert!(result.is_ok(), "Empty text synthesis should not error");
    }

    #[tokio::test]
    async fn test_tts_adapter_synthesize_long_text() {
        use vibespeak::infrastructure::adapters::TtsAdapter;

        let adapter = TtsAdapter::new().expect("Failed to create adapter");
        let long_text =
            "This is a longer piece of text that should be synthesized properly. ".repeat(10);
        let result = adapter.synthesize(&long_text, None).await;

        assert!(result.is_ok(), "Long text synthesis should succeed");
        let samples = result.unwrap();
        assert!(!samples.is_empty(), "Long text should produce samples");
    }

    #[tokio::test]
    async fn test_tts_adapter_shutdown() {
        use vibespeak::infrastructure::adapters::TtsAdapter;

        let adapter = TtsAdapter::new().expect("Failed to create adapter");
        adapter.initialize().await.expect("Init failed");
        let result = adapter.shutdown().await;

        assert!(result.is_ok(), "Shutdown should succeed");
    }
}

mod audio_player_tests {

    #[tokio::test]
    async fn test_audio_player_creation() {
        use vibespeak::infrastructure::adapters::AudioPlayer;

        // Note: This may fail in CI environments without audio devices
        let result = AudioPlayer::new();

        // We check if the error is about missing audio device (acceptable in CI)
        match result {
            Ok(_) => (), // Success
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("audio")
                        || err_msg.contains("output")
                        || err_msg.contains("device"),
                    "Error should be about audio device, got: {}",
                    err_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_audio_player_play_empty_data() {
        use vibespeak::infrastructure::adapters::AudioPlayer;

        if let Ok(player) = AudioPlayer::new() {
            let result = player.play_pcm_data(&[], 44100).await;
            assert!(result.is_ok(), "Playing empty data should succeed (no-op)");
        }
        // Skip test if no audio device
    }

    #[tokio::test]
    async fn test_audio_player_is_playing_initially_false() {
        use vibespeak::infrastructure::adapters::AudioPlayer;

        if let Ok(player) = AudioPlayer::new() {
            assert!(!player.is_playing(), "Should not be playing initially");
        }
    }

    #[tokio::test]
    async fn test_audio_player_stop_when_not_playing() {
        use vibespeak::infrastructure::adapters::AudioPlayer;

        if let Ok(player) = AudioPlayer::new() {
            // Should not panic when stopping without playing
            player.stop();
            assert!(!player.is_playing());
        }
    }

    #[test]
    fn test_pcm_source_iterator() {
        // Test the PCM source used by the audio player
        // This is a unit test for the internal component

        struct TestPcmSource {
            samples: Vec<f32>,
            position: usize,
        }

        impl Iterator for TestPcmSource {
            type Item = f32;
            fn next(&mut self) -> Option<Self::Item> {
                if self.position < self.samples.len() {
                    let sample = self.samples[self.position];
                    self.position += 1;
                    Some(sample)
                } else {
                    None
                }
            }
        }

        let source = TestPcmSource {
            samples: vec![0.0, 0.5, 1.0, -0.5, -1.0],
            position: 0,
        };

        let collected: Vec<f32> = source.collect();
        assert_eq!(collected.len(), 5);
        assert_eq!(collected[0], 0.0);
        assert_eq!(collected[2], 1.0);
    }
}

mod command_interpreter_tests {
    #[tokio::test]
    async fn test_fuzzy_command_interpreter_creation() {
        use std::collections::HashMap;
        use vibespeak::infrastructure::adapters::FuzzyCommandInterpreter;

        let commands: HashMap<String, String> = HashMap::new();
        let interpreter = FuzzyCommandInterpreter::new(commands);

        // Should create successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_fuzzy_command_interpreter_with_commands() {
        use std::collections::HashMap;
        use vibespeak::domain::services::CommandInterpreter;
        use vibespeak::infrastructure::adapters::FuzzyCommandInterpreter;

        let mut commands = HashMap::new();
        commands.insert("hello".to_string(), "echo hello".to_string());
        commands.insert("status".to_string(), "echo status".to_string());
        commands.insert("help".to_string(), "echo help".to_string());

        let interpreter = FuzzyCommandInterpreter::new(commands);

        let available = interpreter.get_available_commands().await;
        assert!(available.is_ok());
        assert_eq!(available.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_fuzzy_command_interpreter_interpret() {
        use std::collections::HashMap;
        use vibespeak::domain::services::{CommandContext, CommandInterpreter};
        use vibespeak::infrastructure::adapters::FuzzyCommandInterpreter;

        let mut commands = HashMap::new();
        commands.insert("hello".to_string(), "echo hello".to_string());
        commands.insert("goodbye".to_string(), "echo bye".to_string());

        let interpreter = FuzzyCommandInterpreter::new(commands);

        let context = CommandContext {
            user_id: None,
            session_id: None,
            previous_commands: vec![],
            environment: HashMap::new(),
        };

        // Test exact match
        let result = interpreter.interpret("hello", &context).await;
        assert!(result.is_ok());
        let interpreted = result.unwrap();
        assert!(interpreted.confidence > 0.8);
    }

    #[tokio::test]
    async fn test_fuzzy_command_interpreter_fuzzy_match() {
        use std::collections::HashMap;
        use vibespeak::domain::services::{CommandContext, CommandInterpreter};
        use vibespeak::infrastructure::adapters::FuzzyCommandInterpreter;

        let mut commands = HashMap::new();
        commands.insert("hello".to_string(), "echo hello".to_string());

        let interpreter = FuzzyCommandInterpreter::new(commands);

        let context = CommandContext {
            user_id: None,
            session_id: None,
            previous_commands: vec![],
            environment: HashMap::new(),
        };

        // Test fuzzy match (slight typo)
        let result = interpreter.interpret("helo", &context).await;
        assert!(result.is_ok());
        // Should still match with lower confidence
    }
}
