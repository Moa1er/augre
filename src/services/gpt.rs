use std::{str::FromStr, time::Duration};
use anyhow::Context;

use chatgpt::prelude::{ChatGPT, ModelConfiguration, ChatGPTEngine};
use url::Url;

use crate::base::types::{HasName, IsEnsurable, Mode, Res, Void};
use crate::services::gpt;

use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};

static NAME: &str = "gpt_sdk";
const URI: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

#[derive(Serialize, Deserialize, Debug)]
struct OAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
struct OAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

pub struct Gpt {
    url: String,
    key: Option<String>,
    mode: Mode,
}

impl HasName for Gpt {
    fn name(&self) -> &'static str {
        NAME
    }
}


impl IsEnsurable for Gpt {
    async fn is_present(&self) -> Res<bool> {
        let _ = self.resolve_key()?;
        
        Ok(true)
    }

    async fn make_present(&self) -> Void {
        Err(anyhow::Error::msg("Cannot perform `make_present`: this should not happen."))
    }
}

impl Gpt {
    pub async fn review(&self, diff: &str, gpt_model: Option<String>) -> Res<String> {
        let model_used: String;

        match gpt_model {
            Some(value) => {
                model_used = value.clone();
                println!("Model used is now: {}", value)
            },
            None => {
                model_used = DEFAULT_MODEL.to_string();
                println!("Default model used: {}", DEFAULT_MODEL.to_string());
            },
        }

        let client = Client::new();
        let openai_key = self.resolve_key()?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", openai_key).parse().unwrap(),
        );

        let prompt_message: Message = Message {
            role: String::from("system"),
            content: String::from("You're a code reviewer or you also comment the code depending on the prompt\n\n"),
        };
        
        let message = REVIEW_PROMPT.replace("{{diff}}", diff);

        let req = OAIRequest {
            model: model_used,
            messages: vec![
                prompt_message,
                Message {
                    role: String::from("user"),
                    content: String::from(message),
                },
            ],
        };

        let res = client
            .post(URI)
            .headers(headers)
            .json(&req)
            .send()
            .await; // Moved the semicolon here
        
        let res = match res {
            Ok(response) => match response.json::<OAIResponse>().await {
                Ok(parsed) => Ok(parsed), // Successful response
                Err(e) => {
                    // Handle JSON parsing error
                    // You can log the error, return it, or take any other action
                    println!("ERROR: probably there is too many tokens sent to chatgpt, try a model with a higher token limit.");
                    Err(e) 
                }
            },
            Err(e) => {
                // Handle HTTP request error
                // You can log the error, return it, or take any other action
                Err(e)
            }
        };
        let message = res?.choices.last().ok_or(anyhow::anyhow!("No choices returned"))?.message.content.clone();
        println!("{}", message);
        // Ok(res.choices[0].message.content.clone())
        Ok(message)
    }

    pub async fn ask(&self, prompt: &str) -> Res<String> {
        let key = self.resolve_key()?;

        let url = format!("{}/v1/chat/completions", self.url);
        let config = ModelConfiguration {
            engine: ChatGPTEngine::Gpt4,
            api_url: Url::from_str(&url)?,
            timeout: Duration::from_secs(300),
            ..Default::default()
        };

        let client = ChatGPT::new_with_config(key, config)?;

        let response = client.send_message(prompt.to_string()).await?;

        Ok(response.message_choices[0].message.content.clone())
    }
}

impl Gpt {
    pub fn new(url: &str, key: &Option<String>, mode: Mode) -> Self {
        Self {
            url: url.to_string(),
            key: key.clone(),
            mode,
        }
    }

    fn resolve_key(&self) -> Res<&str> {
        let key = if self.mode == Mode::OpenAi {
            self.key.as_ref().ok_or(anyhow::Error::msg("OpenAI key not provided.  Please set the `openai_key` config value, or use a local mode."))?
        } else {
            ""
        };

        Ok(key)
    }
}

// Statics.

static REVIEW_PROMPT: &str = r#"
Please perform a code review of the following diff (produced by `git diff` on my code), and provide suggestions for improvement:

```
{{diff}}
```

Please prioritize the response by impact to the code, and please split the suggestions into three categories:
1. Suggestions that pertain to likely runtime bugs or errors.
2. Suggestions that pertain to likely logic bugs or errors.
3. Suggestions that pertain to likely style bugs or errors.

If possible, please also provide a suggested fix to the identified issue.  If you are unable to provide a suggested fix, please provide a reason why.

The format should look like:

```
1. Likely runtime bugs:
- Some suggestion...
- Another...

2. Likely logic bugs:
- Suggestion 1
- Suggestion 2
- Suggestion 3

3. Likely style bugs:
- Suggestion 1
- Suggestion 2
- Suggestion 3
```

For each relevant code snippet, please provide context about where the suggestion is relevant (e.g., `path/file.rs:30`); in addition, if a code snippet would be helpful, please provide a code snippet showing the fix.
"#;