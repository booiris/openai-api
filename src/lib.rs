use thiserror::Error;

#[macro_use]
extern crate derive_builder;

type Result<T> = std::result::Result<T, Error>;

pub mod api {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    /// Container type. Used in the api, but not useful for clients of this library
    #[derive(Deserialize, Debug)]
    pub(crate) struct Container<T> {
        /// Items in the page's results
        pub data: Vec<T>,
    }

    /// Detailed information on a particular model.
    #[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
    pub struct ModelInfo {
        /// The name of the model, e.g. `"davinci"` or `"ada"`
        pub id: String,
        /// The owner of the model. Usually (always?) `"openai"`
        pub owned_by: String,
        ///  Usually (always?) `"model"`
        pub object: String,
    }

    #[derive(Serialize, Debug, Builder, Clone)]
    #[builder(pattern = "immutable")]
    pub struct CompletionArgs {
        /// The id of the model to use for this request
        ///
        /// # Example
        /// ```
        /// # use openai_api::api::CompletionArgs;
        /// CompletionArgs::builder().model("text-davinci-003");
        /// ```
        #[builder(setter(into), default = "\"text-davinci-003\".into()")]
        pub(super) model: String,
        /// The prompt to complete from.
        ///
        /// Defaults to `"<|endoftext|>"` which is a special token seen during training.
        ///
        /// # Example
        /// ```
        /// # use openai_api::api::CompletionArgs;
        /// CompletionArgs::builder().prompt("Once upon a time...");
        /// ```
        #[builder(setter(into), default = "\"<|endoftext|>\".into()")]
        prompt: String,
        /// Maximum number of tokens to complete.
        ///
        /// Defaults to 16
        /// # Example
        /// ```
        /// # use openai_api::api::CompletionArgs;
        /// CompletionArgs::builder().max_tokens(64);
        /// ```
        #[builder(default = "16")]
        max_tokens: u64,
        /// What sampling temperature to use.
        ///
        /// Default is `1.0`
        ///
        /// Higher values means the model will take more risks.
        /// Try 0.9 for more creative applications, and 0 (argmax sampling)
        /// for ones with a well-defined answer.
        ///
        /// OpenAI recommends altering this or top_p but not both.
        ///
        /// # Example
        /// ```
        /// # use openai_api::api::{CompletionArgs, CompletionArgsBuilder};
        /// # use std::convert::{TryInto, TryFrom};
        /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
        /// let builder = CompletionArgs::builder().temperature(0.7);
        /// let args: CompletionArgs = builder.try_into()?;
        /// # Ok::<(), _>(())
        /// # }
        /// ```
        #[builder(default = "1.0")]
        temperature: f64,
        #[builder(default = "1.0")]
        top_p: f64,
        #[builder(default = "1")]
        n: u64,
        #[builder(setter(strip_option), default)]
        logprobs: Option<u64>,
        #[builder(default = "false")]
        echo: bool,
        #[builder(setter(strip_option), default)]
        stop: Option<Vec<String>>,
        #[builder(default = "0.0")]
        presence_penalty: f64,
        #[builder(default = "0.0")]
        frequency_penalty: f64,
        #[builder(default)]
        logit_bias: HashMap<String, f64>,
    }

    impl CompletionArgs {
        /// Build a `CompletionArgs` from the defaults
        #[must_use]
        pub fn builder() -> CompletionArgsBuilder {
            CompletionArgsBuilder::default()
        }
    }

    impl From<&str> for CompletionArgs {
        fn from(prompt_string: &str) -> Self {
            Self {
                prompt: prompt_string.into(),
                ..CompletionArgsBuilder::default()
                    .build()
                    .expect("default should build")
            }
        }
    }

    impl TryFrom<CompletionArgsBuilder> for CompletionArgs {
        type Error = CompletionArgsBuilderError;

        fn try_from(builder: CompletionArgsBuilder) -> Result<Self, Self::Error> {
            builder.build()
        }
    }

    /// Represents a non-streamed completion response
    #[derive(Deserialize, Debug, Clone)]
    pub struct Completion {
        /// Completion unique identifier
        pub id: String,
        /// Unix timestamp when the completion was generated
        pub created: u64,
        /// Exact model type and version used for the completion
        pub model: String,
        /// List of completions generated by the model
        pub choices: Vec<Choice>,
    }

    impl std::fmt::Display for Completion {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.choices[0])
        }
    }

    /// A single completion result
    #[derive(Deserialize, Debug, Clone)]
    pub struct Choice {
        /// The text of the completion. Will contain the prompt if echo is True.
        pub text: String,
        /// Offset in the result where the completion began. Useful if using echo.
        pub index: u64,
        /// If requested, the log probabilities of the completion tokens
        pub logprobs: Option<LogProbs>,
        /// Why the completion ended when it did
        pub finish_reason: String,
    }

    impl std::fmt::Display for Choice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.text.fmt(f)
        }
    }

    /// Represents a logprobs subdocument
    #[derive(Deserialize, Debug, Clone)]
    pub struct LogProbs {
        pub tokens: Vec<String>,
        pub token_logprobs: Vec<Option<f64>>,
        pub top_logprobs: Vec<Option<HashMap<String, f64>>>,
        pub text_offset: Vec<u64>,
    }

    /// Error response object from the server
    #[derive(Deserialize, Debug, Eq, PartialEq, Clone, Error)]
    pub struct ErrorMessage {
        pub message: String,
        #[serde(rename = "code")]
        pub status_code: String,
    }

    impl std::fmt::Display for ErrorMessage {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "err code: {}, err msg: {})",
                self.status_code, self.message
            )
        }
    }

    /// API-level wrapper used in deserialization
    #[derive(Deserialize, Debug)]
    pub(crate) struct ErrorWrapper {
        pub error: ErrorMessage,
    }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
    pub enum ChatRole {
        #[serde(rename = "system")]
        System,
        #[serde(rename = "user")]
        User,
        #[serde(rename = "assistant")]
        Assistant,
    }

    /// The main input is the messages parameter. Messages must be an array of message objects, where each object has a role (either ???system???, ???user???, or ???assistant???) and content (the content of the message). Conversations can be as short as 1 message or fill many pages.
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ChatFormat {
        /// Example
        /// messages=[
        ///     {"role": "system", "content": "You are a helpful assistant."},
        ///     {"role": "user", "content": "Who won the world series in 2020?"},
        ///     {"role": "assistant", "content": "The Los Angeles Dodgers won the World Series in 2020."},
        ///     {"role": "user", "content": "Where was it played?"}
        /// ]
        ///
        /// The system message helps set the behavior of the assistant. In the example above, the assistant was instructed with ???You are a helpful assistant.
        ///
        /// The user messages help instruct the assistant. They can be generated by the end users of an application, or set by a developer as an instruction.
        ///
        /// The assistant messages help store prior responses. They can also be written by a developer to help give examples of desired behavior.
        ///
        #[serde(rename = "role")]
        pub role: ChatRole,
        #[serde(rename = "content")]
        pub content: String,
    }

    impl ChatFormat {
        pub fn new(role: ChatRole, content: String) -> Self {
            Self { role, content }
        }
    }

    impl std::fmt::Display for ChatFormat {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "role: {:?}, content: {}", self.role, self.content)
        }
    }

    #[derive(Serialize, Debug, Builder, Clone)]
    #[builder(pattern = "immutable")]
    pub struct ChatArgs {
        /// The id of the model to use for this request
        /// ID of the model to use. Currently, only gpt-3.5-turbo and gpt-3.5-turbo-0301 are supported.
        ///
        /// # Example
        /// ```
        /// # use openai_api::api::ChatArgs;
        /// ChatArgs::builder().model("gpt-3.5-turbo");
        /// ```
        #[builder(setter(into), default = "\"gpt-3.5-turbo\".into()")]
        pub(super) model: String,
        /// The messages to generate chat completions for.
        ///
        ///
        /// # Example
        /// ```
        /// # use openai_api::api::{ChatArgs,ChatRole,MsgFormat};
        /// ChatArgs::builder().messages(vec![MsgFormat{role: ChatRole::System, content: "You are a helpful assistant.".into()}]);
        /// ```
        #[builder(default)]
        messages: Vec<ChatFormat>,
        /// Maximum number of tokens to complete.
        /// The maximum number of tokens allowed for the generated answer. By default, the number of tokens the model can return will be (4096 - prompt tokens).
        ///
        /// # Example
        /// ```
        /// # use openai_api::api::ChatArgs;
        /// ChatArgs::builder().max_tokens(64);
        /// ```
        #[builder(setter(strip_option), default)]
        max_tokens: Option<u64>,
        /// What sampling temperature to use.
        ///
        /// Default is `1.0`
        ///
        /// Higher values means the model will take more risks.
        /// Try 0.9 for more creative applications, and 0 (argmax sampling)
        /// for ones with a well-defined answer.
        ///
        /// OpenAI recommends altering this or top_p but not both.
        ///
        /// # Example
        /// ```
        /// # use openai_api::api::{ChatArgs, ChatArgsBuilder};
        /// # use std::convert::{TryInto, TryFrom};
        /// # fn main() -> Result<(),Box<dyn std::error::Error>> {
        ///  let builder = ChatArgs::builder().temperature(0.7);
        ///  let args: ChatArgs = builder.try_into()?;
        /// # Ok::<(), _>(())
        /// # }
        /// ```
        #[builder(default = "1.0")]
        temperature: f64,
        #[builder(default = "1.0")]
        top_p: f64,
        /// How many chat completion choices to generate for each input message.
        ///
        /// Defaults to 1
        ///
        #[builder(default = "1")]
        n: u64,
        /// Up to 4 sequences where the API will stop generating further tokens.
        ///
        /// Defaults to null
        ///
        #[builder(setter(strip_option), default)]
        stop: Option<Vec<String>>,
        /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
        ///
        /// Defaults to 0
        ///
        #[builder(default = "0.0")]
        presence_penalty: f64,
        /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.
        ///
        /// Defaults to 0
        ///
        #[builder(default = "0.0")]
        frequency_penalty: f64,
        /// Modify the likelihood of specified tokens appearing in the completion.
        ///
        /// Accepts a json object that maps tokens (specified by their token ID in the tokenizer) to an associated bias value from -100 to 100. Mathematically, the bias is added to the logits generated by the model prior to sampling. The exact effect will vary per model, but values between -1 and 1 should decrease or increase likelihood of selection; values like -100 or 100 should result in a ban or exclusive selection of the relevant token.
        ///
        /// Defaults to null
        #[builder(default)]
        logit_bias: HashMap<String, f64>,
    }

    impl ChatArgs {
        /// Build a `ChatArgsBuilder` from the defaults
        #[must_use]
        pub fn builder() -> ChatArgsBuilder {
            ChatArgsBuilder::default()
        }
    }

    impl From<Vec<(ChatRole, String)>> for ChatArgs {
        fn from(msg: Vec<(ChatRole, String)>) -> Self {
            let msg = msg
                .into_iter()
                .map(|(role, content)| ChatFormat { role, content })
                .collect();
            Self {
                messages: msg,
                ..ChatArgsBuilder::default()
                    .build()
                    .expect("default should build")
            }
        }
    }

    impl TryFrom<ChatArgsBuilder> for ChatArgs {
        type Error = ChatArgsBuilderError;

        fn try_from(builder: ChatArgsBuilder) -> Result<Self, Self::Error> {
            builder.build()
        }
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct ChatAnswer {
        /// Completion unique identifier
        pub id: String,
        /// Unix timestamp when the completion was generated
        pub created: u64,
        /// List of completions generated by the model
        pub choices: Vec<ChatChoice>,
    }

    impl std::fmt::Display for ChatAnswer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.choices[0])
        }
    }

    /// A single completion result
    #[derive(Deserialize, Debug, Clone)]
    pub struct ChatChoice {
        /// The text of the completion. Will contain the prompt if echo is True.
        pub message: ChatFormat,
        /// Offset in the result where the completion began. Useful if using echo.
        pub index: u64,
        /// Why the completion ended when it did
        pub finish_reason: String,
    }

    impl std::fmt::Display for ChatChoice {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.message.fmt(f)
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    /// An error returned by the API itself
    #[error("API returned an Error: {0}")]
    Api(#[from] api::ErrorMessage),
    /// An error the client discovers before talking to the API
    // #[error("Bad arguments: {0}")]
    // BadArguments(String),

    #[error("Build Client arguments: {0}")]
    AsyncProtocol(#[from] reqwest::Error),
}

/// Client object. Must be constructed to talk to the API.
#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    base_url: String,
}

impl Client {
    // Creates a new `Client` given an api token
    #[must_use]
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                .expect("invalid token"),
        );

        Ok(Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
            base_url: "https://api.openai.com/v1/".into(),
        })
    }

    // Allow setting the api root in the tests
    #[cfg(test)]
    fn set_api_root(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }

    /// Private helper for making gets
    async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = &format!("{}{}", self.base_url, endpoint);
        let response = self.client.get(url).send().await?;
        match response.status() {
            reqwest::StatusCode::OK => Ok(response.json::<T>().await?),
            code => {
                let status_code = code.to_string();
                let mut err = response.json::<api::ErrorWrapper>().await?.error;
                err.status_code = status_code;
                Err(Error::Api(err))
            }
        }
    }

    /// Lists the currently available models.
    ///
    /// Provides basic information about each one such as the owner and availability.
    ///
    /// # Errors
    /// - `Error::APIError` if the server returns an error
    pub async fn models(&self) -> Result<Vec<api::ModelInfo>> {
        self.get("models").await.map(|r: api::Container<_>| r.data)
    }

    /// Retrieves an model instance
    ///
    /// Provides basic information about the model such as the owner and availability.
    ///
    /// # Errors
    /// - `Error::APIError` if the server returns an error
    pub async fn model(&self, model: &str) -> Result<api::ModelInfo> {
        self.get(&format!("models/{}", model)).await
    }

    // Private helper to generate post requests. Needs to be a bit more flexible than
    // get because it should support SSE eventually
    async fn post<B, R>(&self, endpoint: &str, body: B) -> Result<R>
    where
        B: serde::ser::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = &format!("{}{}", self.base_url, endpoint);
        let response = self.client.post(url).json(&body).send().await?;
        match response.status() {
            reqwest::StatusCode::OK => Ok(response.json::<R>().await?),
            code => {
                let status_code = code.to_string();
                let mut err = response.json::<api::ErrorWrapper>().await?.error;
                err.status_code = status_code;
                Err(Error::Api(err))
            }
        }
    }

    /// Get predicted completion of the prompt
    ///
    /// # Errors
    ///  - `Error::APIError` if the api returns an error
    pub async fn complete_prompt(
        &self,
        prompt: impl Into<api::CompletionArgs>,
    ) -> Result<api::Completion> {
        let args = prompt.into();
        Ok(self.post("completions", args).await?)
    }

    /// Given a chat conversation, the model will return a chat completion response.
    ///
    /// # Errors
    ///  - `Error::APIError` if the api returns an error
    pub async fn chat(&self, msg: impl Into<api::ChatArgs>) -> Result<api::ChatAnswer> {
        let args = msg.into();
        Ok(self.post("chat/completions", args).await?)
    }
}

#[cfg(test)]
mod unit {

    use mockito::Mock;

    use crate::{
        api::{
            self, ChatAnswer, ChatArgs, ChatFormat, ChatRole, Completion, CompletionArgs, ModelInfo,
        },
        Client, Error,
    };

    fn mocked_client() -> Client {
        let _ = env_logger::builder().is_test(true).try_init();
        Client::new("bogus")
            .unwrap()
            .set_api_root(&format!("{}/", mockito::server_url()))
    }

    #[test]
    fn can_create_client() {
        let _c = mocked_client();
    }

    #[test]
    fn parse_model_info() -> Result<(), Box<dyn std::error::Error>> {
        let example = r#"{
            "id": "ada",
            "object": "model",
            "owned_by": "openai"
        }"#;
        let ei: api::ModelInfo = serde_json::from_str(example)?;
        assert_eq!(
            ei,
            api::ModelInfo {
                id: "ada".into(),
                owned_by: "openai".into(),
                object: "model".into(),
            }
        );
        Ok(())
    }

    fn mock_models() -> (Mock, Vec<ModelInfo>) {
        let mock = mockito::mock("GET", "/models")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
            "object": "list",
            "data": [
              {
                "id": "ada",
                "object": "model",
                "owned_by": "openai"
              }
            ]
          }"#,
            )
            .create();

        let expected = vec![ModelInfo {
            id: "ada".into(),
            owned_by: "openai".into(),
            object: "model".into(),
        }];
        (mock, expected)
    }

    #[tokio::test]
    async fn parse_models() -> crate::Result<()> {
        let (_m, expected) = mock_models();
        let response = mocked_client().models().await?;
        assert_eq!(response, expected);
        Ok(())
    }

    fn mock_model() -> (Mock, api::ErrorMessage) {
        let mock = mockito::mock("GET", "/models/davinci")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "error": {
                    "code": null,
                    "message": "Some kind of error happened",
                    "type": "some_error_type"
                }
            }"#,
            )
            .create();
        let expected = api::ErrorMessage {
            message: "Some kind of error happened".into(),
            status_code: "400".into(),
        };
        (mock, expected)
    }

    #[tokio::test]
    async fn model_error_response() -> crate::Result<()> {
        let (_m, expected) = mock_model();
        let response = mocked_client().model("text-davinci-003").await;
        if let Result::Err(Error::Api(msg)) = response {
            assert_eq!(expected, msg);
        }
        Ok(())
    }

    fn mock_completion() -> crate::Result<(Mock, CompletionArgs, Completion)> {
        let mock = mockito::mock("POST", "/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": "cmpl-uqkvlQyYK7bGYrRHQ0eXlWi7",
                "object": "text_completion",
                "created": 1589478378,
                "model": "davinci:2020-05-03",
                "choices": [
                    {
                    "text": " there was a girl who",
                    "index": 0,
                    "logprobs": null,
                    "finish_reason": "length"
                    }
                ]
                }"#,
            )
            .expect(1)
            .create();
        let args = api::CompletionArgs::builder()
            .model("text-davinci-003")
            .prompt("Once upon a time")
            .max_tokens(5)
            .temperature(1.0)
            .top_p(1.0)
            .n(1)
            .stop(vec!["\n".into()])
            .build()
            .unwrap();
        let expected = api::Completion {
            id: "cmpl-uqkvlQyYK7bGYrRHQ0eXlWi7".into(),
            created: 1589478378,
            model: "davinci:2020-05-03".into(),
            choices: vec![api::Choice {
                text: " there was a girl who".into(),
                index: 0,
                logprobs: None,
                finish_reason: "length".into(),
            }],
        };
        Ok((mock, args, expected))
    }

    // Defines boilerplate here. The Completion can't derive Eq since it contains
    // floats in various places.
    fn assert_completion_equal(a: Completion, b: Completion) {
        assert_eq!(a.model, b.model);
        assert_eq!(a.id, b.id);
        assert_eq!(a.created, b.created);
        let (a_choice, b_choice) = (&a.choices[0], &b.choices[0]);
        assert_eq!(a_choice.text, b_choice.text);
        assert_eq!(a_choice.index, b_choice.index);
        assert!(a_choice.logprobs.is_none());
        assert_eq!(a_choice.finish_reason, b_choice.finish_reason);
    }

    #[tokio::test]
    async fn completion_args() -> crate::Result<()> {
        let (m, args, expected) = mock_completion()?;
        let response = mocked_client().complete_prompt(args).await?;
        assert_completion_equal(response, expected);
        m.assert();
        Ok(())
    }

    fn mock_chat() -> crate::Result<(Mock, ChatArgs, ChatAnswer)> {
        let mock = mockito::mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": "cmpl-uqkvlQyYK7bGYrRHQ0eXlWi7",
                "object": "chat.completion",
                "created": 1589478378,
                "choices": [
                    {
                        "index": 0,
                        "message": {
                          "role": "assistant",
                          "content": "\n\nHello there, how may I assist you today?"
                        },
                        "finish_reason": "stop"
                    }
                ]
                }"#,
            )
            .expect(1)
            .create();
        let args = api::ChatArgs::builder()
            .model("gpt-3.5-turbo")
            .messages(vec![ChatFormat {
                role: ChatRole::System,
                content: "You are a helpful assistant.".into(),
            }])
            .max_tokens(5)
            .temperature(1.0)
            .top_p(1.0)
            .n(1)
            .stop(vec!["\n".into()])
            .build()
            .unwrap();
        let expected = api::ChatAnswer {
            id: "cmpl-uqkvlQyYK7bGYrRHQ0eXlWi7".into(),
            created: 1589478378,
            choices: vec![api::ChatChoice {
                message: ChatFormat {
                    role: ChatRole::Assistant,
                    content: "\n\nHello there, how may I assist you today?".into(),
                },
                index: 0,
                finish_reason: "stop".into(),
            }],
        };
        Ok((mock, args, expected))
    }

    // Defines boilerplate here. The Completion can't derive Eq since it contains
    // floats in various places.
    fn assert_chat_equal(a: ChatAnswer, b: ChatAnswer) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.created, b.created);
        let (a_choice, b_choice) = (&a.choices[0], &b.choices[0]);
        assert_eq!(a_choice.message, b_choice.message);
        assert_eq!(a_choice.index, b_choice.index);
        assert_eq!(a_choice.finish_reason, b_choice.finish_reason);
    }

    #[tokio::test]
    async fn chat_args() -> crate::Result<()> {
        let (m, args, expected) = mock_chat()?;
        let response = mocked_client().chat(args).await?;
        assert_chat_equal(response, expected);
        m.assert();
        Ok(())
    }
}

#[cfg(test)]
mod integration {
    use crate::{
        api::{self, ChatAnswer, Completion},
        Client,
    };

    /// Used by tests to get a client to the actual api
    fn get_client() -> Client {
        let _ = env_logger::builder().is_test(true).try_init();
        let sk = std::env::var("OPENAI_SK").expect(
            "To run integration tests, you must put set the OPENAI_SK env var to your api token",
        );
        Client::new(&sk).unwrap()
    }

    #[tokio::test]
    async fn can_get_models() -> crate::Result<()> {
        let client = get_client();
        client.models().await?;
        Ok(())
    }

    fn assert_model_correct(model_id: &str, info: api::ModelInfo) {
        assert_eq!(info.id, model_id);
        assert_eq!(info.object, "model");
        assert_eq!(info.owned_by, "openai");
    }

    #[tokio::test]
    async fn can_get_model() -> crate::Result<()> {
        let client = get_client();
        assert_model_correct("text-ada-001", client.model("text-ada-001").await?);
        Ok(())
    }

    #[tokio::test]
    async fn complete_string() -> crate::Result<()> {
        let client = get_client();
        client.complete_prompt("Hey there").await?;
        Ok(())
    }

    fn create_args() -> api::CompletionArgs {
        api::CompletionArgsBuilder::default()
            .prompt("Once upon a time,")
            .max_tokens(10)
            .temperature(0.5)
            .top_p(0.5)
            .n(1)
            .logprobs(3)
            .echo(false)
            .stop(vec!["\n".into()])
            .presence_penalty(0.5)
            .frequency_penalty(0.5)
            .logit_bias(maplit::hashmap! {
                "1".into() => 1.0,
                "23".into() => 0.0,
            })
            .build()
            .expect("Bug: build should succeed")
    }

    #[tokio::test]
    async fn complete_explicit_params() -> crate::Result<()> {
        let client = get_client();
        let args = create_args();
        client.complete_prompt(args).await?;
        Ok(())
    }

    fn stop_condition_args() -> api::CompletionArgs {
        api::CompletionArgs::builder()
            .prompt(
                r#"
Q: Please type `#` now
A:"#,
            )
            // turn temp & top_p way down to prevent test flakiness
            .temperature(0.0)
            .top_p(0.0)
            .max_tokens(100)
            .stop(vec!["#".into(), "\n".into()])
            .build()
            .expect("Bug: build should succeed")
    }

    fn assert_completion_finish_reason(completion: Completion) {
        assert_eq!(completion.choices[0].finish_reason, "stop",);
    }

    #[tokio::test]
    async fn complete_stop_condition() -> crate::Result<()> {
        let client = get_client();
        let args = stop_condition_args();
        assert_completion_finish_reason(client.complete_prompt(args).await?);
        Ok(())
    }

    fn stop_chat_args() -> api::ChatArgs {
        api::ChatArgs::builder()
            .messages(vec![api::ChatFormat {
                role: api::ChatRole::System,
                content: "Hello there, how may I assist you today?".into(),
            }])
            // turn temp & top_p way down to prevent test flakiness
            .temperature(0.0)
            .top_p(0.0)
            .max_tokens(100)
            .stop(vec!["#".into(), "\n".into()])
            .build()
            .expect("Bug: build should succeed")
    }

    fn assert_chat_finish_reason(chat: ChatAnswer) {
        assert_eq!(chat.choices[0].finish_reason, "stop",);
    }

    #[tokio::test]
    async fn chat_stop_condition() -> crate::Result<()> {
        let client = get_client();
        let args = stop_chat_args();
        assert_chat_finish_reason(client.chat(args).await?);
        Ok(())
    }
}
