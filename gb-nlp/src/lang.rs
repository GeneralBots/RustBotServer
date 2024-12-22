use gb_core::{Result, Error};
use tracing::instrument;
use whatlang::{Lang, Script, Detector, detect};

pub struct LanguageDetector {
    detector: Detector,
}

impl LanguageDetector {
    pub fn new() -> Self {
        Self {
            detector: Detector::new(),
        }
    }

    #[instrument(skip(self, text))]
    pub fn detect_language(&self, text: &str) -> Result<DetectedLanguage> {
        let info = detect(text)
            .ok_or_else(|| Error::Internal("Failed to detect language".to_string()))?;

        Ok(DetectedLanguage {
            lang: info.lang(),
            script: info.script(),
            confidence: info.confidence(),
        })
    }

    #[instrument(skip(self, text))]
    pub fn is_language(&self, text: &str, lang: Lang) -> bool {
        if let Some(info) = detect(text) {
            info.lang() == lang
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectedLanguage {
    pub lang: Lang,
    pub script: Script,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn detector() -> LanguageDetector {
        LanguageDetector::new()
    }

    #[rstest]
    fn test_detect_english(detector: LanguageDetector) {
        let text = "Hello, this is a test sentence in English.";
        let result = detector.detect_language(text).unwrap();
        assert_eq!(result.lang, Lang::Eng);
        assert!(result.confidence > 0.9);
    }

    #[rstest]
    fn test_detect_spanish(detector: LanguageDetector) {
        let text = "Hola, esta es una prueba en español.";
        let result = detector.detect_language(text).unwrap();
        assert_eq!(result.lang, Lang::Spa);
        assert!(result.confidence > 0.9);
    }

    #[rstest]
    fn test_is_language(detector: LanguageDetector) {
        let text = "Hello world";
        assert!(detector.is_language(text, Lang::Eng));
    }
}
