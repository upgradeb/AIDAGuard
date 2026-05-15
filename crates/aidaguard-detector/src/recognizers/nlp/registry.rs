use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use candle_core::Device;
use candle_nn::{linear, Linear, VarBuilder};

pub struct NlpModel {
    pub bert: candle_transformers::models::bert::BertModel,
    pub classifier: Linear,
    pub tokenizer: tokenizers::Tokenizer,
    pub config: candle_transformers::models::bert::Config,
    pub id_to_label: Vec<String>,
    pub device: Device,
}

impl NlpModel {
    fn load(language: &str) -> Result<Self, anyhow::Error> {
        let (owner, repo_name) = model_info_for_language(language);
        let client = hf_hub::HFClientSync::new()?;
        let repo = client.model(owner, repo_name.to_string());

        let model_path = repo
            .download_file()
            .filename("model.safetensors")
            .send()?;

        let config_path = repo
            .download_file()
            .filename("config.json")
            .send()?;

        let tokenizer_path = repo
            .download_file()
            .filename("tokenizer.json")
            .send()?;

        let device = Device::Cpu;

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        let config_str = std::fs::read_to_string(&config_path)?;

        // Extract num_labels and id2label from config manually (candle Config doesn't have these)
        let raw_config: serde_json::Value =
            serde_json::from_str(&config_str).unwrap_or_default();
        let num_labels = raw_config
            .get("num_labels")
            .and_then(|v| v.as_u64())
            .unwrap_or(9) as usize;
        let id_to_label: Vec<String> = raw_config
            .get("id2label")
            .and_then(|v| {
                let map: HashMap<String, String> = serde_json::from_value(v.clone()).ok()?;
                let mut labels: Vec<(usize, String)> = map
                    .into_iter()
                    .filter_map(|(k, v)| k.parse::<usize>().ok().map(|i| (i, v)))
                    .collect();
                labels.sort_by_key(|(i, _)| *i);
                Some(labels.into_iter().map(|(_, v)| v).collect())
            })
            .unwrap_or_else(default_ner_labels);

        let config: candle_transformers::models::bert::Config =
            serde_json::from_str(&config_str)?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_path], candle_core::DType::F32, &device)?
        };

        let bert = candle_transformers::models::bert::BertModel::load(
            vb.pp("bert"), &config,
        )?;

        let classifier = linear(config.hidden_size, num_labels, vb.pp("classifier"))?;

        Ok(Self {
            bert,
            classifier,
            tokenizer,
            config,
            id_to_label,
            device,
        })
    }
}

fn model_info_for_language(language: &str) -> (&str, &str) {
    match language {
        "zh" => ("ckiplab", "bert-base-chinese-ner"),
        _ => ("dslim", "bert-base-NER"),
    }
}

fn default_ner_labels() -> Vec<String> {
    vec![
        "O".to_string(),
        "B-MISC".to_string(), "I-MISC".to_string(),
        "B-PER".to_string(), "I-PER".to_string(),
        "B-ORG".to_string(), "I-ORG".to_string(),
        "B-LOC".to_string(), "I-LOC".to_string(),
    ]
}

pub struct ModelRegistry {
    models: Mutex<HashMap<String, std::sync::Arc<NlpModel>>>,
}

impl ModelRegistry {
    fn new() -> Self {
        Self { models: Mutex::new(HashMap::new()) }
    }

    pub fn global() -> &'static Self {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<ModelRegistry> = OnceLock::new();
        INSTANCE.get_or_init(ModelRegistry::new)
    }

    pub fn get_or_load(&self, language: &str) -> Result<std::sync::Arc<NlpModel>, anyhow::Error> {
        {
            let cache = self.models.lock().unwrap();
            if let Some(model) = cache.get(language) {
                return Ok(model.clone());
            }
        }

        tracing::info!("Downloading NLP model for language '{}'...", language);
        let model = NlpModel::load(language)?;
        let model = std::sync::Arc::new(model);

        let mut cache = self.models.lock().unwrap();
        cache.insert(language.to_string(), model.clone());

        tracing::info!("NLP model for '{}' loaded successfully", language);
        Ok(model)
    }

    pub fn get_loaded(&self, language: &str) -> Option<std::sync::Arc<NlpModel>> {
        self.models.lock().unwrap().get(language).cloned()
    }

    #[allow(dead_code)]
    pub fn cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.aidaguard.app")
            .join("models")
    }
}
