use gb_core::{Result, Error};
use rust_bert::pipelines::{
    sentiment::SentimentModel,
    ner::NERModel,
    summarization::SummarizationModel,
    question_answering::{QaModel, QuestionAnsweringModel},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{instrument, error};

pub struct TextProcessor {
    sentiment_model: Arc<Mutex<SentimentModel>>,
    ner_model: Arc<Mutex<NERModel>>,
    summarization_model: Arc<Mutex<SummarizationModel>>,
    qa_model: Arc<Mutex<QuestionAnsweringModel>>,
}

impl TextProcessor {
    #[instrument]
    pub async fn new() -> Result<Self> {
        let sentiment_model = SentimentModel::new(Default::default())
            .map_err(|e| Error::Internal(format!("Failed to load sentiment model: {}", e)))?;

        let ner_model = NERModel::new(Default::default())
            .map_err(|e| Error::Internal(format!("Failed to load NER model: {}", e)))?;

        let summarization_model = SummarizationModel::new(Default::default())
            .map_err(|e| Error::Internal(format!("Failed to load summarization model: {}", e)))?;

        let qa_model = QuestionAnsweringModel::new(Default::default())
            .map_err(|e| Error::Internal(format!("Failed to load QA model: {}", e)))?;

        Ok(Self {
            sentiment_model: Arc::new(Mutex::new(sentiment_model)),
            ner_model: Arc::new(Mutex::new(ner_model)),
            summarization_model: Arc::new(Mutex::new(summarization_model)),
            qa_model: Arc::new(Mutex::new(qa_model)),
        })
    }

    #[instrument(skip(self, text))]
    pub async fn analyze_sentiment(&self, text: &str) -> Result<Sentiment> {
        let model = self.sentiment_model.lock().await;
        let output = model.predict(&[text])
            .map_err(|e| Error::Internal(format!("Sentiment analysis failed: {}", e)))?;

        Ok(Sentiment {
            score: output[0].score,
            label: output[0].label.clone(),
        })
    }

    #[instrument(skip(self, text))]
    pub async fn extract_entities(&self, text: &str) -> Result<Vec<Entity>> {
        let model = self.ner_model.lock().await;
        let output = model.predict(&[text])
            .map_err(|e| Error::Internal(format!("Entity extraction failed: {}", e)))?;

        Ok(output[0].iter().map(|entity| Entity {
            text: entity.word.clone(),
            label: entity.entity.clone(),
            score: entity.score,
        }).collect())
    }

    #[instrument(skip(self, text))]
    pub async fn summarize(&self, text: &str) -> Result<String> {
        let model = self.summarization_model.lock().await;
        let output = model.summarize(&[text])
            .map_err(|e| Error::Internal(format!("Summarization failed: {}", e)))?;

        Ok(output[0].clone())
    }

    #[instrument(skip(self, context, question))]
    pub async fn answer_question(&self, context: &str, question: &str) -> Result<Answer> {
        let model = self.qa_model.lock().await;
        let output = model.predict(&[QaModel {
            context,
            question,
        }])
        .map_err(|e| Error::Internal(format!("Question answering failed: {}", e)))?;

        Ok(Answer {
            text: output[0].answer.clone(),
            score: output[0].score,
            start: output[0].start,
            end: output[0].end,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Sentiment {
    pub score: f64,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub text: String,
    pub label: String,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct Answer {
    pub text: String,
    pub score: f64,
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    async fn processor() -> TextProcessor {
        TextProcessor::new().await.unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_sentiment_analysis(processor: TextProcessor) -> Result<()> {
        let text = "I love this product! It's amazing!";
        let sentiment = processor.analyze_sentiment(text).await?;
        assert!(sentiment.score > 0.5);
        assert_eq!(sentiment.label, "positive");
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_entity_extraction(processor: TextProcessor) -> Result<()> {
        let text = "John Smith works at Microsoft in Seattle.";
        let entities = processor.extract_entities(text).await?;
        
        assert!(entities.iter().any(|e| e.text == "John Smith" && e.label == "PERSON"));
        assert!(entities.iter().any(|e| e.text == "Microsoft" && e.label == "ORG"));
        assert!(entities.iter().any(|e| e.text == "Seattle" && e.label == "LOC"));
        
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_summarization(processor: TextProcessor) -> Result<()> {
        let text = "The quick brown fox jumps over the lazy dog. This is a classic pangram that contains every letter of the English alphabet. It has been used for typing practice and font displays for many years.";
        let summary = processor.summarize(text).await?;
        assert!(summary.len() < text.len());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_question_answering(processor: TextProcessor) -> Result<()> {
        let context = "The capital of France is Paris. It is known as the City of Light.";
        let question = "What is the capital of France?";
        
        let answer = processor.answer_question(context, question).await?;
        assert_eq!(answer.text, "Paris");
        assert!(answer.score > 0.8);
        Ok(())
    }
}
