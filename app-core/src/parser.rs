
use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyPage<'a> {
    pub title: Option<&'a str>,
    pub content: &'a str, 
    pub page_number: u32,
}

impl<'a> StudyPage<'a> {
    pub fn new(title: Option<&'a str>, content: &'a str, page_number: u32) -> Self {
        Self { title, content, page_number }
    }
}

pub fn parse_study_text(text: &str) -> Result<Vec<StudyPage<'_>>, ParseError> {
    let mut pages = Vec::new();
    let mut current_title: Option<&str> = None;
    let mut current_page_number = 1;

    let mut content_start_idx: Option<usize> = None;
    let mut content_end_idx: usize = 0;
    
    let mut page_has_data = false;

    for line in text.lines() {
        let trimmed = line.trim();

        let is_t = trimmed.starts_with("T:");
        let is_break = trimmed == "---";

        if is_t || is_break {
            if page_has_data {
                let content = match content_start_idx {
                    Some(start) if start <= content_end_idx => text[start..content_end_idx].trim(),
                    _ => "",
                };
                pages.push(StudyPage::new(current_title, content, current_page_number));
            }

            page_has_data = true;
            content_start_idx = None;
            content_end_idx = 0;

            if is_t {
                let title_text = trimmed[2..].trim();
                current_title = Some(title_text);
            } else if is_break {
                current_title = None;
                current_page_number += 1;
            }
        } else {
            page_has_data = true;
            let line_start = line.as_ptr() as usize - text.as_ptr() as usize;
            let line_end = line_start + line.len();

            if content_start_idx.is_none() {
                content_start_idx = Some(line_start);
            }
            content_end_idx = line_end;
        }
    }

    if page_has_data {
        let content = match content_start_idx {
            Some(start) if start <= content_end_idx => text[start..content_end_idx].trim(),
            _ => "",
        };
        pages.push(StudyPage::new(current_title, content, current_page_number));
    }

    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_multi_page() {
        let input = "T: Page 1\ncontent 1\n---\ncontent 2\nT: Page 3\ncontent 3";
        let pages = parse_study_text(input).unwrap();
        
        assert_eq!(pages.len(), 3);
        
        assert_eq!(pages[0].title, Some("Page 1"));
        assert_eq!(pages[0].content, "content 1");
        assert_eq!(pages[0].page_number, 1);
        
        assert_eq!(pages[1].title, None);
        assert_eq!(pages[1].content, "content 2");
        assert_eq!(pages[1].page_number, 2);
        
        assert_eq!(pages[2].title, Some("Page 3"));
        assert_eq!(pages[2].content, "content 3");
        assert_eq!(pages[2].page_number, 2);
    }
    
    #[test]
    fn test_empty_string() {
        let pages = parse_study_text("").unwrap();
        assert!(pages.is_empty());
    }

    #[test]
    fn test_zero_allocation() {
        let input = String::from("T: Title\nSome content");
        let parsed = parse_study_text(&input).unwrap();
        assert_eq!(parsed[0].title, Some("Title"));
        assert_eq!(parsed[0].content, "Some content");
    }
}