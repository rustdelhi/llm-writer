use std::{convert::Infallible, io::Write, path::PathBuf};

use axum::{response::IntoResponse, Json};
use llm::{InferenceSessionConfig, Model, ModelParameters, OutputRequest};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
enum InferenceError {
    #[error("Failed to load the model: {0}")]
    UnableToLoadModel(String),

    #[error("Failed to perform inference: {0}")]
    UnableToCreateResponse(String),
}

/// Loading llm model
static MODEL: Lazy<Box<dyn llm::Model>> =
    Lazy::new(|| load_model().expect("Unable to load llm model"));

/// Represents a user prompt & corresponding response from LLM.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Prompt {
    prompt: Option<String>,

    #[serde(skip_deserializing)]
    response: Option<String>,
}

impl Prompt {
    /// Gets the prompt string.
    pub fn get_prompt(&self) -> Option<String> {
        self.prompt.clone()
    }

    /// Generates a reply for the given prompt.
    pub fn generate_reply(&mut self) -> impl IntoResponse {
        self.get_prompt().map_or(
            Json(json!({"suggestion": "type something in input box"})),
            |_| -> Json<_> {
                self.infer().expect("Unable to generate LLM response");
                Json(json!({
                "prompt": self.prompt, "response": self.response
                }))
            },
        )
    }

    /// Performs inference based on the prompt and updates the response.
    pub fn infer(&mut self) -> anyhow::Result<()> {
        let start_time = std::time::Instant::now().elapsed().as_millis();
        println!("Model fully loaded! Elapsed: {start_time}ms");

        // ---------- Starting session --------------
        let mut session = MODEL.start_session(InferenceSessionConfig::default());

        let inference_request = &llm::InferenceRequest {
            prompt: (self.prompt.as_deref().unwrap()).into(),
            parameters: &llm::InferenceParameters::default(),
            play_back_previous_tokens: false,
            maximum_token_count: Some(100),
        };

        let mut response = String::new();
        session
            .infer::<Infallible>(
                // Loaded model
                MODEL.as_ref(),
                // Random range
                &mut rand::thread_rng(),
                // Request for inference
                inference_request,
                // Output request
                &mut OutputRequest::default(),
                // Inference response
                |r| match r {
                    llm::InferenceResponse::PromptToken(t)
                    | llm::InferenceResponse::InferredToken(t) => {
                        print!("{t}");
                        response.push_str(&t);

                        std::io::stdout().flush().unwrap();

                        Ok(llm::InferenceFeedback::Continue)
                    }
                    _ => Ok(llm::InferenceFeedback::Continue),
                },
            )
            .map_err(|e| InferenceError::UnableToCreateResponse(e.to_string()))?;

        self.response = Some(response);

        Ok(())
    }
}

/// Loads the model
pub fn load_model() -> anyhow::Result<Box<dyn Model>> {
    let model_architecture = llm::ModelArchitecture::Llama;
    let model_path = PathBuf::from("./assets/open_llama_3b-f16.bin");
    let tokenizer_source = llm::TokenizerSource::Embedded;

    Ok(llm::load_dynamic(
        Some(model_architecture),
        &model_path,
        tokenizer_source,
        ModelParameters::default(),
        llm::load_progress_callback_stdout,
    )
    .map_err(|e| InferenceError::UnableToLoadModel(e.to_string()))?)
}
