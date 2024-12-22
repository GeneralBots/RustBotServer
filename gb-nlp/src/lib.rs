pub mod lang;
pub mod text;

pub use lang::{LanguageDetector, DetectedLanguage};
pub use text::{TextProcessor, Sentiment, Entity, Answer};

#[cfg(test)]
mod tests {
    use super::*;
    use gb_core::Result;

    #[tokio::test]
    async fn test_nlp_integration() -> Result<()> {
        // Initialize NLP components
        let lang_detector = LanguageDetector::new();
        let text_processor = TextProcessor::new().await?;

        // Test language detection
        let text = "This is a test sentence in English.";
        let lang = lang_detector.detect_language(text)?;
        assert_eq!(lang.lang, whatlang::Lang::Eng);

        // Test sentiment analysis
        let sentiment = text_processor.analyze_sentiment(text).await?;
        assert!(sentiment.score > 0.0);

        // Test entity extraction
        let text = "OpenAI released GPT-4 in March 2023.";
        let entities = text_processor.extract_entities(text).await?;

        // Test summarization
        let text = "Artificial intelligence has made significant advances in recent years. Machine learning models can now perform tasks that were once thought to be exclusive to humans. This has led to both excitement and concern about the future of AI.";
        let summary = text_processor.summarize(text).await?;

        // Test question answering
        let context = "Rust is a systems programming language focused on safety and performance.";
        let question = "What is Rust?";
        let answer = text_processor.answer_question(context, question).await?;

        Ok(())
    }
}
